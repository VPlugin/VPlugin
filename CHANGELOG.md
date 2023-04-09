## Release v0.3.0
- Removed `PluginManager::shutdown`, moved all necessary code into the drop implementation.
- VPlugin will no longer reject running in a root environment. It's up to the library user to decide whether
  they want this behavior.
- Removed archive field from `Plugin` struct to reduce memory usage between plugins.
- VPlugin will also generate a `cdylib`. This is mainly a future investment, for a C port and in case building VPlugin becomes too long.
- Added `Shareable` trait to send data into a plugin.
- `Plugin::get_custom_hook()` is now public.
- `PluginManager::get_custom_hook()` and `Plugin::get_custom_hook()` accept anything as a string that implements `AsRef<str>`.
- Plugin metadata is now guaranteed to be loaded while loading the plugin.
- Internal FFI calls now use `CString`. This includes:
  * Entry point names,
  * Function binding (Retrieving function pointers)