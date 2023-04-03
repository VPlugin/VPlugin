/*
 * Copyright 2022-2023 Aggelos Tselios.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0

 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
*/


#![allow(dead_code)]

extern crate libloading;
extern crate log;

use std::env::{self};
use std::ffi::OsStr;
use std::fs::{
        self,
        File
};
use std::mem;
use serde_derive::Deserialize;
use libloading::{
        Library,
        Symbol
};
use zip::ZipArchive;
use crate::VHook;
use crate::error::VPluginError;
use std::io::ErrorKind::{*, self};

/* Personally I believe it looks much better like this */
type LaterInitialized<T> = Option<T>;
macro_rules! initialize_later {
    () => {
        None
    };
}
macro_rules! init_now {
    ($a:expr) => {
        Some($a)
    };
}

/// This is purely for deserialization.
#[derive(Deserialize)]
struct Data {
        metadata: Metadata
}

#[derive(Deserialize)]
struct Metadata {
        description: Option<String>,
        version    : String,
        name       : String,
        objfile    : String
}
/// A struct that represents metadata about
/// a single plugin, like its version and name.
/// 
/// This struct should only be returned by `PluginMetadata::load()`.
/// Otherwise, undefined values will be returned, resulting in undefined
/// behavior.
#[derive(Debug)]
#[repr(C)]
pub struct PluginMetadata {
        pub description: Option<String>,
        pub version    : String,
        pub name       : String,
        pub filename   : String,
        pub objfile    : String
}

/// The plugin type. This is used to identify a single plugin
/// from VPlugin. New plugins should be loaded with `Plugin::load()`,
/// and not be reused explicitly.
#[derive(Debug)]
#[repr(C)]
pub struct Plugin {
        // Metadata about the plugin, will be None if the plugin
        // has not loaded its metadata yet.
        pub metadata       : PluginMetadata,
        pub(crate) filename: String,
        pub(crate) is_valid: bool,
        pub(crate) started : bool,
        pub(crate) raw     : LaterInitialized<Library>,

}

impl PluginMetadata {
        /// Reads a metadata.toml file or returns an error. This is useful
        /// for libraries that wish to make use of VPlugin's internals.
        pub fn read_from_str<T: for<'a> serde::Deserialize<'a>>(string: &str) -> Result<T, VPluginError> {
                let data: T = match toml::from_str(string) {
                        Ok (t) => t,
                        Err(e) => {
                                log::error!("Couldn't read metadata file: {}", e.to_string());
                                return Err(VPluginError::ParametersError)
                        }
                };

                Ok(data)

        }
        
        fn load(plugin: &Plugin) -> Result<Self, VPluginError> {
                let mut plugin_metadata = Self {
                     description: None,
                     version    : String::new(),
                     name       : String::new(),
                     filename   : plugin.filename.clone(),
                     objfile    : String::new(),
                };

                let f = match File::open("metadata.toml") {
                        Ok(val) => val,
                        Err(e) => {
                                match e.kind() {
                                        PermissionDenied => return Err(VPluginError::PermissionDenied),
                                        Unsupported      => return Err(VPluginError::InternalError { err: "Unsupported file".into() }),
                                        NotFound         => return Err(VPluginError::NoSuchFile),
                                        Interrupted      => return Err(VPluginError::InvalidPlugin),
                                        UnexpectedEof    => return Err(VPluginError::InvalidPlugin),
                                        OutOfMemory      => return Err(VPluginError::InternalError { err: "Host is out of memory".into() }),
                                        Other            => return Err(VPluginError::InternalError { err: "Unknown error.".into() }),
                                        _ => panic!()
                                }
                        }
                };

                let contents = match std::io::read_to_string(f) {
                        Ok(contents) => contents,
                        Err(e)        => {
                                log::error!("Error reading metadata string: {}.", e.to_string());
                                return Err(VPluginError::ParametersError);
                        }
                };
                let buffer = String::from(contents.as_str());

                let data_raw: Data = match toml::from_str(&buffer) {
                        Ok(ok) => ok,
                        Err(_) => {
                                return Err(VPluginError::ParametersError)
                        }
                };

                if data_raw.metadata.name.is_empty()
                || data_raw.metadata.name.contains(' ') {
                        /*
                         * Here we panic as without a name, it's impossible to identify the plugin
                         * for future errors.
                         */
                        panic!(
                                "
                                Attempted to use a plugin that has an empty name in its metadata or contains an
                                invalid character in the field.
                                "
                        )
                }

                if data_raw.metadata.version.is_empty()
                || data_raw.metadata.version.contains(' ') {
                        log::error!(
                                "
                                Detected either empty or invalid version string in metadata.toml (Plugin
                                '{}'
                                ", data_raw.metadata.name
                        );
                }

                plugin_metadata.filename = "metadata.toml".to_owned();
                plugin_metadata.version  = data_raw.metadata.version;
                plugin_metadata.name     = data_raw.metadata.name;
                plugin_metadata.objfile  = data_raw.metadata.objfile;

                Ok(plugin_metadata)
        }
}

impl Plugin {
        fn load_archive<S: Copy + Into<String> + AsRef<OsStr>>(filename: S) -> Result<Self, VPluginError> {
                log::trace!("Loading plugin: {}.", &filename.into());
                let tmp = filename.into();
                let fname = std::path::Path::new(&tmp);
                let file = match fs::File::open(fname) {
                        Ok(val) => val,
                        Err(e) => {
                                log::error!(
                                        "Couldn't load {}: {} (error {})",
                                        filename.into(),
                                        e.to_string(),
                                        e.raw_os_error().unwrap_or(0)
                                );
                                match e.kind() {
                                        PermissionDenied => return Err(VPluginError::PermissionDenied),
                                        Unsupported      => return Err(VPluginError::InternalError { err: "Unsupported file".into() }),
                                        NotFound         => return Err(VPluginError::NoSuchFile),
                                        Interrupted      => return Err(VPluginError::InvalidPlugin),
                                        UnexpectedEof    => return Err(VPluginError::InvalidPlugin),
                                        OutOfMemory      => return Err(VPluginError::InternalError { err: "Host is out of memory".into() }),
                                        Other            => return Err(VPluginError::InternalError { err: "Unknown error.".into() }),
                                        _ => panic!()
                                }
                        }
                };
                
                match std::fs::create_dir(env::temp_dir().join("vplugin")) {
                        Err(e) => match e.kind() {
                                ErrorKind::AlreadyExists => (),
                                _ => log::info!("Couldn't create VPlugin directory: {}", e.to_string()),
                        }
                        Ok(_) => env::set_current_dir(env::temp_dir().join("vplugin")).unwrap()
                }

                /* Uncompressing the archive. */
                log::trace!("Uncompressing plugin {}", filename.into());
                let archive = match zip::ZipArchive::new(file) {
                        Ok (v) => v,
                        Err(e) => {
                                log::error!("Archive error: {}. Not extracting plugin.", e.to_string());
                                return Err(VPluginError::InvalidPlugin)
                        }
                };
                Self::extract_archive_files(archive);

                let mut plugin = Self {
                        metadata: unsafe {
                                #[allow(invalid_value)]
                                mem::zeroed() // see below why
                        },
                        raw     : initialize_later!(),
                        filename: filename.into(),
                        is_valid: false,
                        started : false,
                };

                #[allow(deprecated)]
                if let Err(e) = plugin.load_metadata() {
                        return Err(e);
                }
                Ok(plugin)
        }

        fn extract_archive_files(mut archive: ZipArchive<File>) {
                for i in 0..archive.len() {
                        let mut file = archive.by_index(i).unwrap();
                        let outpath = match file.enclosed_name() {
                            Some(path) => path.to_owned(),
                            None => continue,
                        };

                        if (*file.name()).ends_with('/') {
                                fs::create_dir_all(&outpath).unwrap();
                        } else {
                                if let Some(p) = outpath.parent() {
                                        if !p.exists() {
                                            fs::create_dir_all(p).unwrap();
                                        }
                                }
                                
                                let mut outfile = fs::File::create(&outpath).unwrap();
                                std::io::copy(&mut file, &mut outfile).unwrap();
                        }
                }
        }

        /// Loads a plugin into memory and returns it.
        /// After 0.2.0, metadata is also loaded in this call so avoid calling it
        /// again (For your convenience, it has been marked as deprecated).
        pub fn load<S: Copy + Into<String> + AsRef<OsStr>>(filename: S) -> Result<Plugin, VPluginError> {
                let mut plugin = match Self::load_archive(filename) {
                        Err(e) => {
                                log::error!("Couldn't load archive, stopping here.");
                                return Err(e);
                        }
                        Ok (p) => p
                };
                
                /* Until I rewrite the function a little, we shouldn't care about the warning. */
                #[allow(deprecated)]
                match plugin.load_metadata() {
                        Err(e) => {
                                log::error!("Couldn't load metadata, stopping here.");
                                return Err(e);
                        }
                        Ok(_) => {
                                fs::create_dir_all(
                                        env::temp_dir()
                                        .join("vplugin")
                                        .join(&plugin.metadata.name)
                                ).expect("Cannot create plugin directory!");
                        }
                }
                Ok(plugin)
        }

        /// **Executes the plugin.**
        /// 
        /// This function is effectively a standalone replacement for when you want to start
        /// a plugin but don't have access to the global plugin manager.\
        /// There are a series of reasons you probably want to favor the normal
        /// [`PluginManager`](crate::plugin::PluginManager)'s implementation:
        /// * This function **ALWAYS** assumes your plugin's entry point is called `vplugin_init`. Any
        /// other name will simply not work.
        /// * If the plugin has already been started, no checks will be done. Meaning the same plugin will be started
        /// twice.
        /// * Last,
        /// 
        /// In general, this function is intended mainly for test usage and not actual code.
        /// 
        /// ## Example
        /// ```rust
        /// use vplugin::Plugin;
        /// fn main() {
        ///     let mut plugin = Plugin::load("plugin.vpl").unwrap();
        ///     plugin.begin().expect("Error");
        /// }
        /// ```
        pub fn begin(&mut self) -> Result<(), VPluginError> {
                if !self.is_valid {
                        log::error!(
                                "Attempted to start plugin '{}', which is not marked as valid.",
                                self.get_metadata().name
                        );
                        return Err(VPluginError::InvalidPlugin);
                }

                let plugin_entry: Symbol<unsafe extern "C" fn() -> i32>;
                unsafe {
                        plugin_entry = match self.raw
                                        .as_ref()
                                        .unwrap()
                                        .get(b"vplugin_init\0")
                                        {
                                                Ok(fnc) => fnc,
                                                Err(e)  => {
                                                        log::error!(
                                                                "Couldn't initialize plugin: {}",
                                                                e.to_string()
                                                        );
                                                        return Err(VPluginError::FailedToInitialize)
                                                }
                                        };

                        let ___result = plugin_entry();
                        if ___result != 0 {
                                return Err(VPluginError::FailedToInitialize);
                        }
                }
                
                self.started = true;
                Ok(())
        }

        /// Returns a VHook (Generic function pointer) that can be used to exchange data between
        /// your application and the plugin.
        pub(super) fn load_vhook(&self, fn_name: &str) -> Result<VHook, VPluginError> {
                if !self.started || !self.is_valid || self.raw.is_none() {
                        log::error!("Attempted to load plugin function that isn't started or isn't valid");
                        return Err(VPluginError::InvalidPlugin);
                }
                let hook: Symbol<VHook>;
                unsafe {
                        hook = match self.raw
                                .as_ref()
                                .unwrap_unchecked()
                                .get(format!("{}\0", fn_name).as_bytes())
                        {
                            Ok (v) => v,
                            Err(_) => return Err(VPluginError::MissingSymbol),
                        };
                }
                Ok(*hook)
        }

        pub(crate) fn get_hook(&self, fn_name: &str) -> Result<VHook, VPluginError> {
                Self::load_vhook(self, fn_name)
        }

        /// Returns a hook as specified by the generic parameters
        /// 'T' and 'P':
        /// - `T` is the return type of the function representing the hook,
        /// - `P` is the actual function declaration (Don't add `unsafe extern fn`, it's already specified).
        /// The function pointer returned can then be used to exchange data between the server and the plugin.
        pub fn get_custom_hook<P, T>(
                &self,
                fn_name: impl AsRef<str>,
        ) -> Result<unsafe extern fn(P) -> T, VPluginError> {
                if !self.started || !self.is_valid || self.raw.is_none() {
                        log::error!("Cannot load custom hook from non-started or invalid plugin.");
                        return Err(VPluginError::InvalidPlugin);
                }
                let hook: Symbol<unsafe extern fn(P) -> T>;
                unsafe {
                        hook = match self.raw
                                .as_ref()
                                .unwrap_unchecked()
                                .get(format!("{}\0", fn_name.as_ref()).as_bytes())
                        {
                            Ok (v) => v,
                            Err(_) => return Err(VPluginError::MissingSymbol),
                        };
                }
                Ok(*hook)
        }

        /// A function to load the plugin's metadata into
        /// the plugin. In order to access the plugin's metadata,
        /// use the [get_metadata](crate::plugin::Plugin::get_metadata) function.
        /// See also: [PluginMetadata](crate::plugin::PluginMetadata)
        #[deprecated = "The plugin's metadata will be automatically loaded along with the plugin itself."]
        pub fn load_metadata(&mut self) -> Result<(), VPluginError> {
                match PluginMetadata::load(self) {
                        Ok (v) => {
                                let plugin_dir_name = env::temp_dir()
                                        .join("vplugin")
                                        .join(&v.name);

                                fs::create_dir_all(&plugin_dir_name).unwrap();
                                fs::copy(&v.objfile, plugin_dir_name.join(&v.objfile)).unwrap();

                                self.raw       = unsafe {
                                        init_now!(Library::new(plugin_dir_name.join(&v.objfile)).unwrap())
                                };
                                self.is_valid = true;
                                self.metadata = v;

                                Ok(())
                        },
                        Err(e) => {
                                log::error!("Couldn't load metadata ({}): {}", self.filename, e.to_string());
                                Err(e)
                        }
                }
        }

        /// Returns a reference to the plugin metadata, if loaded.
        /// Otherwise, `None` is returned.
        pub fn get_metadata(&self) -> &PluginMetadata {
                &self.metadata
        }

        /// Unloads the plugin, if loaded and started,
        /// calling its destructor in the process and
        /// freeing up resources.
        /// 
        /// ## `Err` returned:
        /// If an `Err` value was returned, this means that
        /// the plugin was either not loaded, invalid, doesn't
        /// have a destructor function. In that case, you can try
        /// using [`Plugin::force_terminate`](crate::plugin::Plugin::force_terminate)
        /// to force the plugin to be removed, risking safety and undefined behavior.
        pub fn terminate(&mut self) -> Result<(), VPluginError> {
                if self.raw.is_none() {
                        return Err(VPluginError::InvalidPlugin);
                }

                if !self.started {
                        log::error!("Cannot terminate a plugin that wasn't started in the first place.");
                        return Err(VPluginError::InvalidPlugin);
                }

                let destructor: Symbol<unsafe extern "C" fn() -> ()>;
                unsafe {
                        destructor = match self.raw
                                .as_ref()
                                .unwrap_unchecked()
                                .get(b"vplugin_exit\0")
                        {
                            Ok (v) => v,
                            Err(_) => {
                                log::warn!(
                                        target: "Destructor",
                                        "Plugin {} does not have a destructor. Force terminate if needed.",
                                        self.get_metadata().name
                                );
                                return Err(VPluginError::InvalidPlugin)
                            },
                        };

                        destructor();
                }

                self.started  = false;
                if cfg!(feature = "non_reusable_plugins") {
                        self.is_valid = false;
                        self.raw      = None;
                        self.filename = String::new();
                }
                Ok(())
        }

        /// ###### *Returns whether the function specified is available on the plugin.*
        /// 
        /// **Deprecated**: This function has been replaced with [Plugin::is_symbol_present](crate::plugin::Plugin::is_symbol_present).
        #[deprecated = "Replaced by Plugin::is_symbol_present which is more accurate and safer."]
        pub fn is_function_available(&self, name: &str) -> bool {
                if self.raw.is_none() {
                        log::warn!("Avoid using misinitialized plugins as properly loaded ones (Missing shared object file).");
                        return false;
                }
                unsafe {
                        self.raw.as_ref().unwrap().get::<unsafe extern "C" fn()>(name.as_bytes()).is_ok()
                }
        }

        /// ### Returns whether the requested symbol is present in the plugin implementation.
        /// 
        /// This function returns a boolean that indicates whether a symbol requested (That is, a global
        /// variable or a function) is available to the caller.
        /// 
        /// **NOTE: The symbol is not checked as to whether it has the same type as the one requested.
        /// For example, if symbol a has type `i32` but you request a function symbol named `a`, this function
        /// will most likely still return true. This is because at runtime, types are not available, and
        /// VPlugin does not yet test if the symbol is callable (A function).**
        /// 
        /// ## Example
        /// ```
        /// use vplugin::Plugin;
        /// let plugin = Plugin::load("file.vpl").unwrap();
        /// 
        /// if plugin.is_symbol_present<fn(i32, i32) -> u8>("myfunc") {
        ///     /* Symbol present. */
        /// } else {
        ///     /* Symbol not present. */
        /// }
        /// ```
        /// 
        /// ## Panics
        /// Panics if `fn_name` contains invalid encoding or `self` has not yet been properly initialized.
        pub fn is_symbol_present<T, S>(&self, fn_name: S) -> bool
        where
                S: Sized + Into<String>
        {
                unsafe {
                        self.raw
                                .as_ref()
                                .unwrap()
                                .get::<T>(fn_name.into().as_bytes())
                                .is_ok()
                }
        }
}

impl Drop for Plugin {
        fn drop(&mut self) {
                let plugin_dir_name = env::temp_dir()
                        .join("vplugin")
                        .join(&self.metadata.name);

                match std::fs::remove_dir_all(&plugin_dir_name) {
                        Err(e) => {
                                log::warn!(
                                        "Couldn't remove directory '{}' corresponding to plugin '{}': {}",
                                        plugin_dir_name.display(),
                                        self.metadata.name,
                                        e.to_string()
                                )
                        },
                        Ok(_) => ()
                }
        }
}
