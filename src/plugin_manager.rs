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
use std::{ffi::{c_void, c_int}, env};
use libloading::Symbol;
use crate::error::VPluginError;

use super::plugin::Plugin;

/// ## PluginManager
/// The plugin manager is responsible for managing all loaded plugins,
/// like deploying them, attaching hooks and executing them.
/// 
/// While it could be technically possible to avoid its usage, all
/// plugins expect some alternative helper for usual tasks.
/// The PluginManager is responsible for all tasks not involving plugins
/// (Yes, even unloading plugins from memory) and should be a core part
/// of your application.
#[repr(C)]
pub struct PluginManager {
        plugin : Vec<Plugin>,
        entry  : String,
        running: bool,
        errcode: u32
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
                #[cfg(unix)] /* Windows applications often need admin rights. */
                if is_superuser::is_superuser() {
                        panic!("VPlugin may not be run")
                }
                Self {
                        plugin : Vec::new(),
                        entry  : String::from("vplugin_init"),
                        running: false, /* No plugins running */
                        errcode: 0
                }
        }

        /// Loads a plugin through PluginManager. This function calls Plugin::load(filename)
        /// under the hood, so you can also use it.
        /// 
        /// See also: [register_plugin](PluginManager::register_plugin).
        pub fn load_plugin(&mut self, filename: &str) -> Result<Plugin, VPluginError> {
                Plugin::load(filename)
        }

        /// Registers a plugin into the PluginManager.
        /// 
        /// This step will be useful if you want to automatically remove plugins
        /// when they exit before your application, or if you need to leave your
        /// plugin idle, and automatically detect any errors.
        pub fn register_plugin(&mut self, plugin: Plugin) -> Result<(), VPluginError> {
                self.plugin.push(plugin);
                Ok(())
        }

        /// Sets the name of a plugin's entry point.
        /// 
        /// You probably want to set this to something unique to your application,
        /// like `appname_init`.
        pub fn set_entry_point(&mut self, entry_point: &str) {
                let entry_point_with_null = &format!("{}\0", entry_point);
                self.entry = String::from(entry_point_with_null)
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
                &mut self,
                plugin: &Plugin,
                hook: &str
        ) -> Result<unsafe extern fn(P) -> T, VPluginError> {
                plugin.get_custom_hook(hook)
        }
        
        /// **Executes the entry point of the plugin.**
        /// 
        /// This function is used to execute the entry point of the plugin,
        /// effectively starting the plugin like a normal executable.
        pub fn begin_plugin(&mut self, plugin: &mut Plugin) -> Result<(), VPluginError>{
                if !plugin.is_valid {
                        log::error!(
                                "Attempted to start plugin '{}', which is not marked as valid.",
                                plugin.get_metadata().as_ref().unwrap().name
                        );
                        return Err(VPluginError::InvalidPlugin);
                }

                if plugin.started {
                        log::error!(
                                "Plugin '{}' has already been initialized.",
                                plugin.get_metadata().as_ref().unwrap().name
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
                                log::error!("Couldn't start plugin: Entry point '{}' did not return success", self.entry);
                                return Err(VPluginError::FailedToInitialize);
                        }
                }
                plugin.started = true;
                Ok(())
        }

        /// ## Shutdown the PluginManager
        /// This function is used to shutdown the plugin manager,
        /// by removing all loaded plugins, neutralizing its state
        /// and making some quick cleanup.
        /// 
        /// ## Unloading Plugins
        /// By default, the plugin manager will try to unload all plugins
        /// normally, by calling its function destructor. If that fails,
        /// then the failed plugin will be forced to unload itself,
        /// which may cause undefined behavior.
        /// 
        /// ## Comparing this function with `impl Drop for PluginManager`
        /// This function cannot implement the `Drop` trait because it takes
        /// ownership of the plugin manager, to ensure that the plugin manager
        /// will not be accidentally reused (Use after free). It does call
        /// `drop` on the plugin manager though automatically.
        pub extern fn shutdown(mut self) {
                for plugin in self.plugin.iter_mut() {
                        plugin.terminate().unwrap_or_else(|_| log::warn!("Error occured while unloading plugin."));
                }
        }
}

impl Default for PluginManager {
        fn default() -> Self {
                Self::new()
        }
}

impl Drop for PluginManager {
        fn drop(&mut self) {
            let vplugin_dir = env::temp_dir().join("vplugin");
            for plug in &mut self.plugin {
                plug
                        .terminate()
                        .unwrap_or_else(|e|
                                log::error!("Couldn't unload plugin (VPlugin Error): {}", e.to_string())
                        );
                drop(plug);
            }
            match std::fs::remove_dir_all(&vplugin_dir) {
                Ok(()) => log::trace!("Removed directory: {}", vplugin_dir.display()),
                Err(e) => {
                        log::warn!(
                                "Couldn't remove VPlugin: {} (err {}). No cleanup will be performed.",
                                e.to_string(),
                                e.raw_os_error().unwrap()
                        )
                }
            }
        }
}