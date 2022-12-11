<div align="right">
        Last edited on Nov. 09 2022. <br>
        This file specifies the acceptable format for VPlugin-compatible plugins (modules). <br>
        Version: v1.0.0
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
```
- The `plugin.so` file:\
        - This file is the actual object file that will be loaded as the plugin. The standards it should follow are specified in [Section 3](#3-shared-object-format).

## 2. Archiving Format
Plugins that need to be compatible with VPlugin shall be created as a non-encrypted, (preferably) low-compression ZIP archive. Usually any archiving utility (Such as `zip`) will be able to create such an archive.

## 3. Shared Object Format
The raw shared object file that will be used to interact between the plugin and the actual application / library should NOT have a `main` function, but rather follow the application's guidelines for the entry point. As a fallback, a function named `vplugin_init` can be created, but should be avoided as no constructor exists. 

It should also be built with the ability to dynamically load it as a shared library, and its symbols should not be mangled (At least the entry point and the destructor). Last, for plugins that are written in the Rust programming language, a C linkage / ABI must be specified. This is often done by specifying `extern "C"`, although Cargo projects may as well specify `cdylib` as the crate type.

## 4. File Extensions
Plugins compatible with VPlugin are expected to use the `.vpl` file extension, to be forward compatible with future versions of VPlugin (Which may allow to specify filenames without extensions). This file extension is to be used on the final archive, so a compiled plugin should be named `plugin.vpl`.