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

/// # Shareable
/// When you use this trait, you can send data to your plugins in an easier way. The trait
/// defines two functions to send data.
/// The generic parameter `T` can be used.
/// For safety reasons, all the data you send and receive must implement `Send` and `Sync`.
/// As the plugin may internally create new threads, it's important to ensure runtime safety
/// by using these traits.
/// 
/// # Example
/// ```
/// use vplugin::Shareable;
/// 
/// pub struct Data {
///     something: i32,
///     something_else: String
/// }
/// 
/// impl Shareable for Data {
///     fn send(&mut self, plugin: &vplugin::Plugin) {
///           let mut attacher = plugin.get_hook::<(), data: &mut Self>("plugin_attach_data").expect("Can't locate hook");
///           attacher(self);
///     }
///     unsafe fn send_ptr(ptr: *mut Self, plugin: &crate::Plugin) {
///         panic!("Try to limit this function's use inside Rust!")
///     }
/// }
/// ```
pub trait Shareable<T>
where
    T: Send + Sync + ?Sized
{
    /// Sends `self` into the plugin.
    fn send(&mut self, plugin: &crate::Plugin);
    
    /// Sends `self` as a pointer (`ptr`) to the plugin given.
    /// This function is marked `unsafe` because pointer dereferencing
    /// and sizes are
    unsafe fn send_ptr(ptr: *mut Self, plugin: &crate::Plugin);
}