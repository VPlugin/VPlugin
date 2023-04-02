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


#![warn(clippy::all)]
#![allow(clippy::needless_doctest_main)]

//! # VPlugin
//! VPlugin is a cross-platform plugin framework for Rust. VPlugin takes care of your project's
//! plugin part so you can focus on the actual application without having to worry about the
//! details of your plugins.
//! 
//! # Example
//! First, creating a skeleton app for the plugin:
//! `main.rs`:
//! 
//! ```rust
//! extern crate vplugin;
//! use vplugin::PluginManager;
//! 
//!
//! const FILENAME: &str = "plugin/example.vpl";
//! 
//! fn main() {
//!     let manager    = PluginManager::new();
//!     let mut plugin = manager.load_plugin(FILENAME).expect("Couldn't load plugin");
//! 
//!     manager.set_entry_point("app_entry");
//! 
//!     manager.begin_plugin().expect("Couldn't begin plugin");
//!     if plugin.terminate().is_err() {
//!             unsafe { plugin.force_terminate(); }
//!             plugin_manager.shutdown();
//!     };
//! }
//! ```
//! Then, create a new plugin with [vplugin-init](https://github.com/VPlugin/vplugin-init/):
//! ```text
//! $ vplugin-init \
//!     --directory plugin \
//!     --name "example-plugin" \
//!     --version "0.1.0" \
//!     --language rust \
//!     --objfile plugin.obj
//! 
//! $ cd plugin/
//! ```
//! Afterwards, create an entry point and a destructor for the plugin:
//! `plugin.rs`:
//! ```rust
//! /* Entry point */
//! #[no_mangle]
//! fn app_entry()-> i32 {
//!     println!("Hello plugin!");
//!     0
//! }
//! 
//! /* Destructor */
//! #[no_mangle]
//! fn vplugin_exit() {
//!     println!("Goodbye plugin!");
//! }
//! ```
//! Then package the plugin as a whole with [vplugin-package](https://github.com/VPlugin/vplugin-package/):
//! ```text
//! $ vplugin-package -o example.vpl
//! $ cd .. # To get back to the application
//! ```
//! Now, running the application should give you two messages to stdout:
//! ```text
//! $ cargo r --release
//! 
//! Hello plugin!
//! Goodbye plugin!
//! ```

/* I am still working on the C/C++ part. */
#![allow(improper_ctypes_definitions)]

mod plugin;
mod plugin_manager;
mod error;
mod shareable;

/// Reexports of VPlugin's types.
pub use plugin_manager::*;
pub use plugin::*;
pub use shareable::Shareable;

/// Reexporting libloading to assist projects that need the library.
pub use libloading;