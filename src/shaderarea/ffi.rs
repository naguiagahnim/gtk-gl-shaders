use super::new_area_for_shader;
use glib::translate::IntoGlibPtr;
use gtk::prelude::*;
use std::ffi::CStr;

#[no_mangle]
pub extern "C" fn gtk_gl_shaders_new_area_for_shader(
    shader: *const std::os::raw::c_char,
) -> *mut gtk::ffi::GtkWidget {
    let _ = gtk::init();
    let shader_str = unsafe { CStr::from_ptr(shader).to_string_lossy().into_owned() };
    let area = new_area_for_shader(shader_str, vec![]);
    let widget: gtk::Widget = area.upcast();
    unsafe { widget.into_glib_ptr() }
}
