<div align="right">
        Last edited on Dec. 23 2022. <br>
        This file specifies the acceptable format for VPlugin-compatible plugins (modules). <br>
        Version: v1.0.1
</div>

# VPlugin -- Plugin Format Specification
[Preamble](#0-preamble)\
[Directory Structure](#1-directory-structure)\
[Archiving Format](#2-archiving-format)\
[Shared Object Format](#3-shared-object-format)\
[File Extensions](#4-file-extensions)

## 0. Preamble
This file declares the official requirements for a file to be considered a VPlugin-compatible plugin,
also referred to as a module or package. VPlugin is an open source plugin system written in the Rust programming language, and this document specifies whether a plugin can be considered compatible with VPlugin.

### Definitions
- A plugin will be referred sometimes as a VPlugin package, or VPlugin module. Any understandable convention may be used.
- VPlugin, unless specified otherwise, refers purely to the Rust crate (The library implementation) and not to any other tools (Such as `vplugin-package`).
## 1. Directory Structure
An extracted (uncompressed) directory can have any sorts of formats (Even source code) as long as the following files are available after extraction:
- The `metadata.toml` file:\
        -  This file contains information about the plugin, like its name, version and license identifier. It should do so inside a table named `metadata`.\
        An example can be seen here:
```toml
# metadata.toml
[metadata]
name    = "ExamplePlugin"
version = "1.4.5"
objfile = "plugin.so"

description = "Your plugin's description. (Totally optional)."
```
Available fields include:
- `name` - The name of the plugin (Required) **(Empty strings not allowed!)**
- `version` - The version of the plugin (Required) **(Empty strings not allowed!)**
- `objfile` - The file that VPlugin should use to look up functions (Required since 1.0.1) **(Empty strings not allowed!)**
- `description` - The plugin's description (Optional)

- The `objfile` as specified in the `metadata.toml` file:
        - It's the actual plugin file with the functions and globals that will be used. For compatibility,
        you can use the `raw.so` file (Which was used previously), however you can use any file name you
        wish to use. A nice example would be `plugin.obj` (The `obj` file extension just signifies it's not human-readable; You can use any extension you wish).

## 2. Archiving Format
Plugins that need to be compatible with VPlugin shall be created as a non-encrypted, (preferably) low-compression ZIP archive. Usually any archiving utility (Such as `zip`) will be able to create such an archive. Any compression algorithm can be used.

VPlugin provides tools both to extract and compress VPlugin packages.

## 3. Shared Object Format
The raw shared object file that will be used to interact between the plugin and the actual application / library should NOT have a `main` function, but rather follow the application's guidelines for the entry point. As a fallback, a function named `vplugin_init` can be created. However compatibility with the application the plugin is targeting is not guaranteed.

It should also be built with the ability to dynamically load it as a shared library, and its symbols should not be mangled (At least the entry point and the destructor). Last, for plugins that are written in the Rust programming language, a C linkage / ABI must be specified. This is often done by specifying `extern "C"`, although Cargo projects may as well specify `cdylib` as the crate type.

## 4. File Extensions
Plugins compatible with VPlugin are expected to use the `.vpl` file extension, to be forward compatible with future versions of VPlugin (Which may allow to specify filenames without extensions). This file extension is to be used on the final archive, so a compiled plugin should be named `plugin.vpl`.