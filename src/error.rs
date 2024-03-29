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

extern crate thiserror;
use thiserror::Error;

/// ## **Generic error code enum**
/// 
/// This enum represents possible errors that can occur while using
/// VPlugin. They are usually an `Err` value on a `Result` enum returned
/// by the API's functions.
/// 
/// ## Error Handling
/// If a function from VPlugin returned an `Err` with this enum, then you are
/// advised to see what the error is (There is a `#derive(Debug)` also used there).
/// If an `InternalError` is returned, then take a look at the `String` parameter instead.
#[derive(Error, Debug)]
#[repr(C)]
pub enum VPluginError {
        /// Invalid parameters passed to the function,
        /// only useful for FFI calls.
        #[error("Passed invalid parameters")]
        ParametersError,
        /// The plugin given is not valid
        /// for this operation.
        #[error("Invalid plugin requested")]
        InvalidPlugin,
        /// The file requested is not available.
        #[error("File requested couldn't be accessed")]
        NoSuchFile,
        /// You do not have permission to access something
        /// on the host system.
        #[error("Access denied on requested permissions")]
        PermissionDenied,
        /// The symbol requested is not present in the raw
        /// object file.
        #[error("Symbol requested is not present in the plugin")]
        MissingSymbol,
        /// The plugin failed to initialize.
        #[error("Plugin failed to initialize")]
        FailedToInitialize,
        /// Internal error: See the `String` parameter
        /// to determine what the error is.
        #[error("Internal error: {err:?}")]
        InternalError {err: String},
}