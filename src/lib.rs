use libloading::os::unix::Library;
use std::{ptr, sync::Once};

mod shader_area;

pub use shader_area::ShaderArea;

static INIT: Once = Once::new();

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
    });
}
