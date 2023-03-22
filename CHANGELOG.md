## Release v0.3.0 (Unreleased)
- Removed `PluginManager::shutdown`, moved all necessary code into the drop implementation.
- VPlugin will no longer reject running in a root environment. It's up to the library user to decide whether
  they want this behavior.
- Removed archive field from `Plugin` struct to reduce memory usage between plugins.