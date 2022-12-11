/*
 * Copyright 2022 Aggelos Tselios.
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

extern crate libloading;
extern crate log;

use std::env;
use std::fs::{
        self,
        File
};
use std::path::Path;
use std::process::abort;
use serde_derive::Deserialize;
use libloading::{
        Library,
        Symbol
};
use zip::ZipArchive;
use crate::VHook;
use crate::error::VPluginError;
use std::io::ErrorKind::*;

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
}

/// The plugin type. This is used to identify a single plugin
/// from VPlugin. New plugins should be loaded with `Plugin::load()`,
/// and not be reused explicitly.
#[derive(Debug)]
#[repr(C)]
pub struct Plugin {
        // Metadata about the plugin, will be None if the plugin
        // has not loaded its metadata yet.
        pub metadata       : Option<PluginMetadata>,
        pub(crate) filename: String,
        pub(crate) is_valid: bool,
        pub(crate) started : bool,
        pub(crate) raw     : Option    <Library>,
        pub(crate) archive : ZipArchive<File>,

}

impl PluginMetadata {
        fn load(plugin: &Plugin) -> Result<Self, VPluginError> {
                log::debug!("Loading metadata for plugin: {}", plugin.filename);
                let mut plugin_metadata = Self {
                     description: None,
                     version    : String::new(),
                     name       : String::new(),
                     filename   : plugin.filename.clone(),
                };

                let _ = match File::open("metadata.toml") {
                        Ok(val) => val,
                        Err(e) => {
                                match e.kind() {
                                        PermissionDenied => return Err(VPluginError::PermissionDenied),
                                        Unsupported      => return Err(VPluginError::InternalError("Unsupported file".into())),
                                        NotFound         => return Err(VPluginError::NoSuchFile),
                                        Interrupted      => return Err(VPluginError::InvalidPlugin),
                                        UnexpectedEof    => return Err(VPluginError::InvalidPlugin),
                                        OutOfMemory      => return Err(VPluginError::InternalError("Host is out of memory".into())),
                                        Other            => return Err(VPluginError::InternalError("Unknown error.".into())),
                                        _ => panic!()
                                }
                        }
                };

                let buffer = String::new();

                let data_raw: Data = match toml::from_str(&buffer) {
                        Ok(ok) => ok,
                        Err(_) => abort()
                };

                plugin_metadata.filename = "metadata.toml".to_owned();
                plugin_metadata.version  = data_raw.metadata.version;
                plugin_metadata.name     = data_raw.metadata.name;

                Ok(plugin_metadata)
        }
}

impl Plugin {
        /// ## The function to load a plugin from VPlugin.
        /// This is the most useful function for VPlugin: It will load the plugin into
        /// a struct and return it (Or simply fail to do so, in which case an `Err` value will be
        /// returned instead as a [VPluginError](VPluginError)).
        /// ## Possible Errors
        /// As loading a plugin is a pretty unsafe operation (Although still handled within the
        /// function itself to save you some time), you are advised to carefully call this function
        /// and avoid simply `unwrap`ing the `Result` passed. This will also help to avoid panics
        /// due to poor error handling.
        pub fn load(filename: &str) -> Result<Plugin, VPluginError> {
                log::debug!("Attempting to load {} as a plugin.", filename);
                let fname = std::path::Path::new(filename);
                let file = match fs::File::open(fname) {
                        Ok(val) => {
                                log::debug!("Successfully loaded {} as a plugin.", filename);
                                val
                        }
                        Err(e) => {
                                log::error!("Couldn't load {}: {}.", filename, e.to_string());
                                match e.kind() {
                                        PermissionDenied => return Err(VPluginError::PermissionDenied),
                                        Unsupported      => return Err(VPluginError::InternalError("Unsupported file".into())),
                                        NotFound         => return Err(VPluginError::NoSuchFile),
                                        Interrupted      => return Err(VPluginError::InvalidPlugin),
                                        UnexpectedEof    => return Err(VPluginError::InvalidPlugin),
                                        OutOfMemory      => return Err(VPluginError::InternalError("Host is out of memory".into())),
                                        Other            => return Err(VPluginError::InternalError("Unknown error.".into())),
                                        _ => panic!()
                                }
                        }
                };

                /* 
                 * First we need to change to the temporary
                 * directory and then uncompress the archive. 
                 * Otherwise we fill the current directory with the contents
                 * of the archive when we shouldn't.
                 */
                env::set_current_dir(Path::new("/tmp")).expect("Failed to switch to the temporary directory");

                /* Uncompressing the archive. */
                let mut archive = zip::ZipArchive::new(file).unwrap();
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

                let raw;
                unsafe {
                        raw = match Library::new("./raw.so") {
                                Ok (v) => v,
                                Err(_) => {
                                        return Err(VPluginError::InvalidPlugin)
                                }
                        }

                }
                let plugin = Self {
                        metadata: None,
                        filename: String::from(filename),
                        is_valid: false,
                        started : false,
                        raw     : Some(raw),
                        archive,
                };
                Ok(plugin)
        }

        pub(super) fn load_vhook(&self, fn_name: &str) -> Result<VHook, VPluginError> {
                let hook: Symbol<VHook>;
                unsafe {
                        hook = match self.raw
                                .as_ref()
                                .unwrap_unchecked() /* No problem, already pretty unsafe */
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

        pub(crate) fn get_custom_hook<P, T>(
                &self,
                fn_name: &str
        ) -> Result<unsafe extern fn(P) -> T, VPluginError> {
                let hook: Symbol<unsafe extern fn(P) -> T>;
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

        /// A function to load the plugin's metadata into
        /// the plugin. In order to access the plugin's metadata,
        /// use the [get_metadata](crate::plugin::Plugin::get_metadata) function.
        pub fn load_metadata(&mut self) -> Result<(), VPluginError> {
                match PluginMetadata::load(self) {
                        Ok (v) => {
                                self.is_valid = true;
                                self.metadata = Some(v);
                                Ok(())
                        },
                        Err(e) => {
                                Err(e)
                        }
                }
        }

        /// Returns a reference to the plugin metadata, if loaded.
        /// Otherwise, `None` is returned.
        pub fn get_metadata(&self) -> &Option<PluginMetadata> {
                &self.metadata
        }

        /// Unloads the plugin, if loaded and started,
        /// calling its destructor in the process and
        /// freeing up resources.
        /// 
        /// ## `Err` returned:
        /// If an `Err` value was returned, this means that
        /// the plugin was either not loaded, invalid, doesn't
        /// have a destructor function.
        pub fn terminate(&self) -> Result<(), VPluginError> {
                if self.raw.is_none() {
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
                            Err(_) => return Err(VPluginError::InvalidPlugin),
                        };

                        destructor();
                }
                Ok(())
        }

        pub fn force_terminate(&mut self) {
        }

        pub extern fn is_function_available(&self, name: &str) -> bool {
                if self.raw.is_none() {
                        return false;
                }
                unsafe {
                        self.raw.as_ref().unwrap().get::<unsafe extern "C" fn()>(name.as_bytes()).is_ok()
                }
        }

        /// Returns whether the plugin metadata is available
        /// and loaded. You can use this to avoid unwrap()'ing
        /// on invalid values.
        #[inline(always)]
        pub fn is_metadata_loaded(&self) -> bool {
                self.metadata.is_some()
        }
}
