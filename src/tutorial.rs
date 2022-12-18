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

//! # VPlugin Tutorial
//! VPlugin is a framework: It requires you to get familiar with the way it works, and this is exactly
//! what this page will help you to do. If you ever get stuck, read this page again: Things may actually
//! make sense when you read a simpler documentation than the default one.
//! 
//! # 1. The plugin itself.
//! VPlugin expects your plugin to follow some specific layout conventions to be accepted. If this is not the
//! case, then VPlugin will refuse to load the plugin no matter what. You can read the actual specification
//! in [this file](https://github.com/VPlugin/VPlugin/blob/master/spec/PluginFormat.md), but to simplify,
//! you basically need to add a `metadata.toml` file, which should look like this:
//! ```toml
//! [metadata]
//! name = "your-plugin" # Your plugin's name
//! version = "0.23" # The version of your plugin
//! objfile = "plugin.obj" # The object file that will be loaded. We'll get into this soon.
//! ```
//! And a shared object file, which contains the actual symbols for your plugin. The `objfile` field
//! from the former file sets the filename of the object file, so you should put that into it. A good filename
//! recommendation would be something like `plugin.obj`, although you can use anything.
//! 
//! Writing your plugin requires you to explicitly specify a C linkage, and disable mangling. For Rust,
//! this can be done by using the `#[no_mangle]` attribute and `extern "C" for your declarations. Example:
//! ```rust
//! #[no_mangle]
//! extern "C" fn callable(x: i32);
//! ```
//! While for C / C++, this can be done just with `extern "C"`:
//! ```
//! extern "C" {
//!     int some_fn() {
//!             return 0;
//!     }
//! }
//! ```
//! You can use any language you wish, as long as the plugin you are writing has an API for that language.
//! Usually, this is not necessary hard, you can also port the types on your own by splitting the plugin into
//! two languages.
//! 
//! # 2. The host process
//! The project that your plugin is targeting will be referred to as the host process. It doesn't matter whether
//! the actual OS process is not the original one (eg. It was forked), this naming is just for convenience.
//! In order to load plugins, you must first create a `PluginManager`, which is used to operate on plugins loaded
//! from VPlugin:
//! ```rust
//! fn main() {
//!     let mut manager = PluginManager::new();
//! }
//! ```
//! You also probably want to set a different entry point for your plugin that `vplugin_init`. In that case, you can
//! use `PluginManager::set_entry_point()`:
//! ```rust
//! manager.set_entry_point("plugin_entry");
//! ```
//! From there on, you should somehow get a filename of a plugin to load. This can be through argument parsing, some file
//! dialog that you use, some predefined location that all plugins are stored, or anything else. For now, we will just
//! declare the plugin's filename as a variable:
//! ```
//! const PLUGIN_FILE_NAME: &str = "/home/user/excellent_plugin.vpl";
//! ```
//! Loading and starting the plugin are seperated, because an application may wish to load multiple plugins at startup,
//! but only start them after considering that it's safe to do so. So, first we have to load the plugins. We can either
//! use `Plugin::load()` (Constructor for plugins), or the plugin manager itself:
//! ```rust
//! let mut plugin = manager
//!                 .load_plugin(PLUGIN_FILE_NAME)
//!                 .expect("Couldn't load plugin");
//! ```
//! Since we don't have a reason to start them later, we can just start them right on:
//! ```rust
//! manager.begin_plugin(&plugin).expect("Couldn't start plugin");
//! ```
//! `PluginManager::begin_plugin` takes a reference to the plugin to start and will fail if the entry point didn't return
//! 0 on the function end or if the plugin is somehow impossible to be started. For example, if the plugin wasn't properly
//! loaded.
//! 
//! Finally, unloading the plugin should be handled so it doesn't consume any more resources. Here's how to do it:
//! ```rust
//! if plugin.terminate().is_err() {
//!     /* 
//!      * The plugin couldn't be properly terminated for some reason. When that
//!      * happens but you still want to immediately unload the plugin, you should use
//!      * PluginManager::force_terminate() instead.
//!      *
//!      * Keep in mind that this function will take ownership of the plugin and then drop it,
//!      * which means it cannot be reused afterwards.
//!      */
//!     manager.force_terminate(plugin);
//! }
//! ```
//! You can also drop the plugin functionality entirely at some point, by shutting off the plugin manager. It's not required
//! but you can do this for a "safe-mode" implementation:
//! ```rust
//! manager.shutdown();
//! ```
//! # 3. First actual application
//! We will write an application that will include an "OpenGL" plugin. Because OpenGL in Rust is not exactly
//! the easiest thing to do, we will use plain C, GLFW and GLEW for the plugin. The application itself will pretty
//! much do nothing but print a message when we try to run it, unless we pass the `--opengl-plugin` flag to it
//! which will make it load the plugin.
//! 
//! First, we will write the basics of the application. I've decided to use `clap` for the command line parsing, just
//! copy and paste all this so you don't waste any time:
//! 
//! ```rust
//! extern crate clap;
//! extern crate vplugin;
//! 
//! use clap::{
//!     arg,
//!     Command
//! };
//! 
//! fn main() {
//!     /* Whether to load the OpenGL Plugin. By default we won't. */
//!     let mut load_opengl_plugin = false;
//! 
//!     let matches = Command::new("very-cool-app")
//!             .version("0.1.0")
//!             .about("A simple command line utility to create a base VPlugin module.")
//!             .arg(
//!                     arg!(--opengl-plugin)
//!                     .help("Whether to load the OpenGL plugin or not.")
//!                     .default_value(false)
//!             )
//!             .get_matches();
//!     
//!     load_opengl_plugin = matches::get_one::<String>("opengl-plugin");
//!     if opengl_plugin.is_none() {
//!             eprintln!("Please use the --opengl-plugin flag!");
//!             return;
//!     } else {
//!             app()
//!     }
//! }
//! 
//! fn app() -> ! {
//!     std::process::exit(0);
//! }
//! ```
//! The code above is simply parsing values from the command line and if --opengl-plugin is specified
//! then `app()` will be called. So our entire code from now on will be written inside `app()`.
//! 
//! First, we have to instanciate the plugin manager, as specified above:
//! ```rust
//! fn app() -> ! {
//!     let plugin_manager = vplugin::PluginManager::new();
//!     plugin_manager.set_entry_point("gl_init")
//! 
//!     std::process::exit(0);
//! }
//! ```
//! Then, we will have to create the plugin itself. In a new folder (Can be a subfolder as well),
//! create a C file and a `metadata.toml` file. As an exercise, add the contents of the latter yourself,
//! specifically the `name`, `version` and `objfile` fields.
//! 
//! In the C file, first include the headers of each library. In this case:
//! ```
//! #include <stdlib.h>
//! #include <GL/glew.h>
//! #include <GLFW/glfw3.h>
//! ```
//! Your main function then will have to initialize those libraries and then create an OpenGL Context.
//! You can do this in any way you want, but because this is just a tutorial, you can just copy and paste
//! the following code:
//! ```
//! int gl_init() {
//!     if (glfwInit() != true) {
//!             puts("Couldn't initialize GLFW");
//!             return -1;
//!     }
//! 
//!     GLFWwindow* window = glfwCreateWindow(1280, 720, "OpenGL Example", NULL, NULL);
//!     if (!window) {
//!             puts("Couldn't create GLFW window");
//!             glfwTerminate();
//!             return -1;
//!     }
//! 
//!     glfwMakeContextCurrent(window);
//!     if (glewInit() != GLEW_OK) {
//!             puts("Couldn't initialize GLEW.");
//!             glfwDestroyWindow(window);
//!             glfwTerminate();
//!             return -1;
//!     }
//! 
//!     uint32_t major = 0;
//!     glClearColor(1.0f, 0.0f, 0.0f, 1.0f);
//!     glGetIntegerV(GL_MAJOR, &major); /* Just a test, not actually needed. */
//! 
//!     while (!glfwWindowShouldClose(window)) {
//!             glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
//! 
//!             glfwSwapBuffers(window);
//!             glfwPollEvents();
//!     }
//!     return 0;
//! }
//! 
//! void vplugin_exit() {
//!     glewTerminate();
//!     glfwMakeContextCurrent(NULL);
//!     glfwTerminate();
//! }
//! ```
//! This code is the basic code along with the error handling to show a window on the
//! screen with a red color. You can also add anything else relating to OpenGL. Important
//! part here is to notice that the plugin and the actual application will run under the same thread,
//! which may not be your best practice. We also added the plugin destructor to properly free resources
//! when we are done with the plugin.
//! 
//! Now, the weird part: VPlugin does have a tool to create plugins and package them, but it's still a
//! WIP so we will use the standard utilities instead. But first, we have to compile our plugin (We will use
//! GCC, any compiler should work):
//! ```
//! $ gcc your-file.c -o plugin.obj -fno-exceptions -O3
//! ```
//! This will output our object file as `plugin.obj`. Feel free to change this to match your `metadata.toml`
//! file.
//! Finally, we have to package our plugin as a ZIP archive:
//! ```
//! $ zip plugin.zip plugin.obj metadata.toml
//! ```
//! This is everything for our plugin. Now we have to write the code to actually use it.
//! In the Rust application, load and start the plugin like this:
//! ```
//! let mut plugin = Plugin::load("/path/to/plugin.zip").expect("Couldn't find plugin");
//! plugin_manager.begin_plugin(&plugin).expect("Failed to start the OpenGL plugin.");
//! ```
//! Finally, once the plugin is finished, we should terminate it:
//! ```
//! if plugin.terminate().is_err() {
//!     eprintln!("Failed to terminate the OpenGL plugin.");
//!     plugin.force_terminate();
//! }
//! ```
//! And this is it! Running this application should give you a window with a red background.
//! For more documentation, see the `docs.rs` uploads.