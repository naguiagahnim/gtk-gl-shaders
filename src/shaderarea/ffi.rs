use super::new_area_for_shader;
use glib::object::Cast;
use glib::translate::IntoGlibPtr;
use std::ffi::CStr;
use std::ptr;

fn init_opengl() {
    let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();

    epoxy::load_with(|name| {
        unsafe { library.get::<_>(name.as_bytes()) }
            .map(|symbol| *symbol)
            .unwrap_or(ptr::null())
    });
}

#[no_mangle]
pub extern "C" fn gtk_gl_shaders_new_area_for_shader(
    shader: *const std::os::raw::c_char,
) -> *mut gtk::ffi::GtkWidget {
    let _ = gtk::init();
    init_opengl();
    let shader_str = unsafe { CStr::from_ptr(shader).to_string_lossy().into_owned() };
    let area = new_area_for_shader(shader_str, vec![]);
    let widget: gtk::Widget = area.upcast();
    unsafe { widget.into_glib_ptr() }
}
