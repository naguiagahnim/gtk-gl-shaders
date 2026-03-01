use glib::{GlibLogger, GlibLoggerDomain, GlibLoggerFormat};
use libloading::os::unix::Library;
use log::LevelFilter;
use std::{ptr, sync::Once};

mod shader_area;

pub use shader_area::ShaderArea;

static INIT: Once = Once::new();
static GLIB_LOGGER: GlibLogger =
    GlibLogger::new(GlibLoggerFormat::Plain, GlibLoggerDomain::CrateTarget);

fn init() {
    INIT.call_once(|| {
        // gtk need to be initialized for most operations
        gtk::init().expect("GTK initialization failed");

        // libepoxy needs to be initialized before calling any opengl function
        let library = unsafe { Library::new("libepoxy.so.0") }.expect("Can't find libepoxy.so.0");
        epoxy::load_with(|name| {
            unsafe { library.get::<_>(name.as_bytes()) }
                .map(|symbol| *symbol)
                .unwrap_or(ptr::null())
        });

        // initialize the `log` crate to forward to glib's logger
        let _ = log::set_logger(&GLIB_LOGGER);
        log::set_max_level(LevelFilter::Debug);
    });
}
