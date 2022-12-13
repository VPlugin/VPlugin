<div align="center">
        <img src="assets/logo.svg" width="244"></img>
        <h2>VPlugin: A plugin framework for Rust.</h2>
        <img src="https://img.shields.io/crates/v/vplugin?style=flat-square">
        <img src="https://img.shields.io/docsrs/vplugin?label=Documentation&style=flat-square">
        <img src="https://img.shields.io/github/license/VPlugin/VPlugin?style=flat-square">
        <img src="https://github.com/VPlugin/VPlugin/actions/workflows/default.yml/badge.svg">
        <br>
        <a href="javascript:void(0)">Website</a> |
        <a href="https://github.com/AndroGR/VPlugin/issues">Issues</a> |
        <a href="https://docs.rs/crate/vplugin">Documentation</a>
        <p style="padding-top: 12px;">
                VPlugin is a Rust framework to develop and use plugins on applications and libraries, <br> including but not limited to games, text editors, command line and graphical applications.
        </p>
        <br>
</div>

## But why?
I found myself ever since starting out programming struggling to find a proper solution to creating a plugin API without copying other's code. Eventually, I decided to write my own library, which ended up becoming a fully-featured set of tools that would allow me to easily write a plugin system from scratch without having to do the same thing 1000 times.

Generally, VPlugin aims to become a low-level block in your application, where you are going to build everything else on top of. VPlugin will **not** create the plugin system for you; It will do the dirty under-the-hood work and give you a high-level abstraction over it.

## MSRV
VPlugin officially supports only the latest **stable** version of the Rust language. You may be able to get it to compile on a few older versions, but do not be confused if your computer blows up or you get a ton of error messages on the console.

## VPlugin's Features:
- High performance thanks to Rust's blazingly fast implementation.
- Straightforward and minimal: Won't get in your way.
- Multi-threaded: Allows you to run everything on another thread. Excludes plugins, which may use code that isn't multi-thread compatible.

## Supported Languages
Generally, most compiled languages will be supported, as long as they can build as a shared object file (shared library). This means that while VPlugin itself is Rust-only for now, it's perfectly possible to write a plugin usable by VPlugin in C, C++ or even Vala. See [the Plugin Specification](./spec/PluginFormat.md) for more details. Key requirement here is a way to export your types to those languages, which requires giving off safety guarantees and a lot of expertise.