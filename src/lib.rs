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

//! # VPlugin
//! VPlugin is a cross-platform Rust framework to develop and use plugins on any sort of project.
//! It offers a suite of tools and libraries that make it easy to integrate with
//! any sort of project. VPlugin aims to provide a high-level abstraction for applications
//! that cannot afford to reinvent the whee;
//! 
//! # Using VPlugin
//! VPlugin is a large project. In order to learn how to use it, you are advised to read
//! [the VPlugin Guide](https://vplugin.github.io/getting-started.html). You can also
//! read the documentation provided here to get to learn the API.
//! 
//! # Loading plugins
//! Before you load a plugin, you will have to instanciate a `PluginManager`:
//! ```
//! use vplugin::PluginManager;
//! let mut manager = PluginManager::new();
//! manager.set_entry_point("plugin_init"); /* Entry point for your plugin */
//!
//! let mut plugin = manager.load_plugin("/path/to/plugin").expect("Couldn't load plugin.");
//! plugin.load_metadata().expect("Invalid plugin metadata.");
//!```
//! # Supported platforms:
//! VPlugin supports the following platforms:
//! - Windows (Only missing `raw.so` replacement.)
//! - MacOS X (Only missing `raw.so` replacement.)
//! - GNU/Linux (Complete)
//! - FreeBSD (Complete)

/* I am still working on the C/C++ part. */
#![allow(improper_ctypes_definitions)]

extern crate zip;
extern crate libloading;
extern crate toml;

mod plugin;
mod plugin_manager;
mod error;

/// Reexports of VPlugin's types.
pub use plugin_manager::*;
pub use plugin::*;
