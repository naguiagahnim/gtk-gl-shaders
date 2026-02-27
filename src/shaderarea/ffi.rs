use crate::shaderarea::TextureData;

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

fn load_texture(path: &str) -> Option<TextureData> {
    let img = image::open(path).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Some(TextureData {
        width: width as i32,
        height: height as i32,
        data: img.into_raw(),
    })
}

#[no_mangle]
pub extern "C" fn gtk_gl_shaders_new_area_for_shader(
    shader: *const std::os::raw::c_char,
    texture_paths: *const *const std::os::raw::c_char,
    texture_count: i32,
) -> *mut gtk::ffi::GtkWidget {
    let _ = gtk::init();
    init_opengl();

    let shader_str = unsafe { CStr::from_ptr(shader).to_string_lossy().into_owned() };

    let textures: Vec<TextureData> = unsafe {
        (0..texture_count)
            .filter_map(|i| {
                let path_ptr = *texture_paths.add(i as usize);
                let path = CStr::from_ptr(path_ptr).to_string_lossy();
                load_texture(&path)
            })
            .collect()
    };

    let area = new_area_for_shader(shader_str, textures);
    let widget: gtk::Widget = area.upcast();
    unsafe { widget.into_glib_ptr() }
}
