## Release v0.3.0 (Unreleased)

- The plugin manager will now hold a reference to all plugins loaded, regardless of whether you call
  `PluginManager::register_plugin`
- Removed `PluginManager::shutdown`, moved all necessary code into the drop implementation.
- The plugin manager now has a `'plug` lifetime, due to the addition of references within the struct.
- VPlugin will no longer reject running in a root environment. It's up to the library user to decide whether
  they want this behavior.
- Removed archive field from `Plugin` struct to reduce memory usage between plugins.