//! GTK4 GLArea widget with custom GLSL shaders for GJS/JavaScript.
//!
//! This library exposes GTK4 `GLArea` widgets with custom GLSL shaders to
//! GJS/JavaScript via GObject Introspection. It handles the OpenGL boilerplate
//! including fullscreen quad rendering, shader compilation, texture uploads,
//! and automatic cleanup.
//!
//! # Usage
//!
//! The library is designed to be used through GObject Introspection from GJS.
//! See the [ShaderArea] type for the main widget implementation.
//!
//! # Initialization
//!
//! The library automatically initializes libepoxy on first use.
//! This is handled internally by the [`init()`] function.

use glib::{GlibLogger, GlibLoggerDomain, GlibLoggerFormat};
use libloading::os::unix::Library;
use log::LevelFilter;
use std::{ptr, sync::Once};

mod shader_area;

pub use shader_area::ShaderArea;

/// Global initialization guard - ensures one-time setup of OpenGL.
static INIT: Once = Once::new();

/// GLIB logger that forwards Rust log messages to GLib's logging system.
static GLIB_LOGGER: GlibLogger =
    GlibLogger::new(GlibLoggerFormat::Plain, GlibLoggerDomain::CrateTarget);

/// Initializes the library: libepoxy, and logging.
///
/// This function is called automatically by FFI entry points and should not
/// need to be called manually. It ensures:
///
/// - libepoxy is loaded for OpenGL function resolution
/// - The Rust `log` crate is configured to forward to GLib's logger
fn init() {
    INIT.call_once(|| {
        // Load libepoxy for OpenGL function resolution
        let library = unsafe { Library::new("libepoxy.so.0") }.expect("Can't find libepoxy.so.0");
        epoxy::load_with(|name| {
            unsafe { library.get::<_>(name.as_bytes()) }
                .map(|symbol| *symbol)
                .unwrap_or(ptr::null())
        });

        // Configure Rust logging to forward to GLib
        let _ = log::set_logger(&GLIB_LOGGER);
        log::set_max_level(LevelFilter::Debug);
    });
}
