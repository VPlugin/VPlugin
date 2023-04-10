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
//! Note that VPlugin is **NOT** writing your program's API. That's something you will have to handle
//! manually.
//! 
//! # Example
//! First, creating a skeleton app for the plugin:
//! `**main.rs**`:
//! 
//! ```rust
//! extern crate vplugin;
//! use vplugin::PluginManager;
//! use std::path::PathBuf;
//! 
//! fn main() {
//!     let plugin_path = PathBuf::from("/path/to/your/plugin.vpl");
//!     let mut plugin_manager = PluginManager::new();
//!     plugin_manager.set_entry_point("app_entry");
//! 
//!     let mut plugin = plugin_manager.load(plugin_path).expect("Plugin cannot be loaded!");
//!     plugin_manager.begin_plugin(&mut plugin).expect("Plugin couldn't be started!");
//! }
//!
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
//! unsafe extern "C" fn app_entry() -> i32 {
//!     println!("Hello plugin!");
//!     0
//! }
//! 
//! /* Destructor. Will alwa */
//! #[no_mangle]
//! unsafe extern "C" fn vplugin_exit() {
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
pub mod shareable; // Are you happy `rustc`?

use std::process::{Termination, ExitCode};

use error::VPluginError;

/// Reexports of VPlugin's types.
pub use plugin_manager::*;
pub use plugin::*;
pub use shareable::Shareable;

/// Reexporting libloading to assist projects that need the library.
pub use libloading;

/// Result type for VPlugin.
/// 
/// This type is used within VPlugin as an alternative to the default
/// `Result`. The main reason for this change is to always return a
/// [`VPluginError`](crate::error::VPluginError) as an error, but allow any
/// type to be returned as a success value,
/// similar to how [`io::Result`](std::io::Result) works.
/// 
/// ## Examples
/// ```
/// extern crate vplugin;
/// 
/// // vplugin::Result implements Termination
/// fn main() -> vplugin::Result<()> {
///     vplugin::Result::Ok(())
/// }
/// ```
pub enum Result<T> {
    Ok(T),
    Err(VPluginError)
}

impl Termination for crate::Result<()> {
    fn report(self) -> std::process::ExitCode {
        ExitCode::SUCCESS
    }
}

impl<T> Result<T> {
    /// Returns the `Ok` value, or panics if `self` is `Err`.
    /// 
    /// `self` will be consumed after this call.
    pub fn unwrap(self) -> T {
        match self {
            Self::Ok(t) => t,
            Self::Err(e) => {
                panic!("Attemptd to Result::unwrap() an Err value: ", &e);
            }
        }
    }
}