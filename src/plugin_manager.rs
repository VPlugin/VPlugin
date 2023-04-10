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

extern crate libloading;
use std::{ffi::{c_void, c_int, CString, OsStr}, env, fs};
use libloading::Symbol;
use crate::error::VPluginError;

use super::plugin::Plugin;

/// ## PluginManager
/// A `PluginManager` is responsible for managing all loaded plugins,
/// like deploying them, attaching hooks, cleaning up the filesystem, etc.
/// You should have it as a singleton instance in your application.
/// 
#[repr(C)]
pub struct PluginManager {
        entry: CString
}

/// ## VHook
/// The `VHook` is a type to represent a generic function by VPlugin.
/// There is only a generic parameter available, a standard `void*`
/// which can then be translated into the actual struct (Expected to
/// be provided by your application or library). The return value is always
/// an integer, to indicate success or failure. To save data, you should make
/// a field available to the struct you pass.
/// ## Safety
/// Generally, using void pointers is unsafe by miles, and you should avoid it.
/// If you can, use Rust alternatives instead, such as generics or simply getting
/// the functions yourself.
/// In any case, you should only use VHook if you are ready to deal with type mismatches
/// and a ton of other issues.
pub type VHook = unsafe extern "C" fn(*mut c_void) -> c_int;

impl PluginManager {
        /// Creates a new, empty PluginManager and returns it.
        pub fn new() -> Self {
                let dir = env::temp_dir().join("vplugin");
                fs::create_dir(dir).expect("Unable to create VPlugin directory.");
                
                Self {
                        entry  : CString::new("vplugin_init").expect("CString::new error")
                }
        }

        /// Loads a plugin through PluginManager. This function calls Plugin::load(filename)
        /// under the hood, so you can also use it.
        /// 
        /// ## Parameters
        /// * `filename` A path to the plugin to load.
        /// 
        /// ## Panics
        /// May panic if `filename` is not a valid string.
        pub fn load_plugin<P: Copy + Into<String> + AsRef<OsStr>>(&mut self, filename: P) -> Result<Plugin, VPluginError> {
                if filename.into().is_empty() {
                        return Err(VPluginError::ParametersError)
                }
                Plugin::load(filename)
        }

        /// **This function is no longer relevant, it's only kept for compatibility.**
        #[deprecated(since = "v0.3.0", note = "This function is no longer relevant, it's only kept for compatibility.")]
        pub fn register_plugin(&mut self, _plugin: &mut Plugin) -> Result<(), VPluginError> {
                Ok(())
        }

        /// Sets the name of a plugin's entry point.
        /// 
        /// You probably want to set this to something unique to your application,
        /// like `appname_init`.
        pub fn set_entry_point(&mut self, entry_point: &str) {
                self.entry = CString::new(entry_point).expect("CString::new error")
        }

        /// Returns a hook from the plugin specified.
        /// See [VHook](crate::plugin_manager::VHook) for more information.
        pub fn get_hook(&mut self, plugin: &Plugin, hook: &str) -> Result<VHook, VPluginError> {
                plugin.get_hook(hook)
        }

        /// Returns a hook as specified by the generic parameters
        /// 'T' and 'P':
        /// - `T` is the return type of the function representing the hook,
        /// - `P` is the actual function declaration (Don't add `unsafe extern fn`, it's already specified).
        /// The function pointer returned can then be used to exchange data between the server and the plugin.
        pub fn get_custom_hook<P, T>(
                &self,
                plugin: &Plugin,
                hook: impl AsRef<str>,
        ) -> Result<unsafe extern fn(P) -> T, VPluginError> {
                plugin.get_custom_hook(hook)
        }
        
        /// **Executes the entry point of the plugin.**
        /// 
        /// This function is used to execute the entry point of the plugin,
        /// effectively starting the plugin like a normal executable.
        pub fn begin_plugin(&mut self, plugin: &mut Plugin) -> Result<(), VPluginError> {
                if !plugin.is_valid {
                        log::error!(
                                "Attempted to start plugin '{}', which is not marked as valid.",
                                plugin.get_metadata().name
                        );
                        return Err(VPluginError::InvalidPlugin);
                }

                if plugin.started {
                        log::error!(
                                "Plugin '{}' has already been initialized.",
                                plugin.get_metadata().name
                        );
                        return Err(VPluginError::FailedToInitialize);
                }

                let plugin_entry: Symbol<unsafe extern "C" fn() -> i32>;
                unsafe {
                        plugin_entry = match plugin.raw
                                        .as_ref()
                                        .unwrap()
                                        .get(self.entry.as_bytes())
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
                                log::error!("Couldn't start plugin: Entry point '{}' did not return success", self.entry.as_c_str().to_string_lossy());
                                return Err(VPluginError::FailedToInitialize);
                        }
                }

                plugin.started = true;
                Ok(())
        }
}

impl Drop for PluginManager {
        fn drop(&mut self) {
            let vplugin_dir = env::temp_dir().join("vplugin");
            match std::fs::remove_dir_all(&vplugin_dir) {
                Ok(()) => log::trace!("Removed directory: {}", vplugin_dir.display()),
                Err(e) => {
                        log::warn!(
                                "Couldn't remove {}: {} . No cleanup will be performed.",
                                vplugin_dir.display(),
                                e.to_string(),
                        )
                }
            }
        }
}