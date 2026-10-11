//! `keyboard` is a small library for reading keyboard input on Windows.
//!
//! See the crate-level documentation for an overview and examples, or jump
//! straight to the [`windows`] module which contains the whole implementation.

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// The Windows implementation of the `keyboard` API.
///
/// This module is the entire library: the [`Key`] enum and three functions,
/// [`pressed_keys`], [`read_char`], and [`wait_for_key`]. Everything is also
/// re-exported at the crate root, so `use keyboard::{Key, read_char};` works
/// directly.
#[cfg(target_os = "windows")]
#[path = "../os/windows.rs"]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(not(target_os = "windows"))]
compile_error!(
    "the `keyboard` crate currently implements Windows support only; \
     unix/linux backends are not implemented yet"
);