use super::new_area_for_shader;
use crate::shaderarea::TextureData;
use glib::object::Cast;
use glib::translate::IntoGlibPtr;
use std::ffi::CStr;
use std::ptr;

/// Loads epoxy's OpenGL function pointers from the system libepoxy.
///
/// This needs to happen before any GL call, and specifically before GTK's
/// GLArea tries to render. When the library is loaded by GJS, GTK is already
/// initialized but epoxy hasn't been pointed at the right symbols yet — this
/// call fixes that.
fn init_opengl() {
    let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();
    epoxy::load_with(|name| {
        unsafe { library.get::<_>(name.as_bytes()) }
            .map(|symbol| *symbol)
            .unwrap_or(ptr::null())
    });
}

/// Reads an image from disk and converts it to raw RGBA8 pixels.
///
/// Returns `None` if the file doesn't exist or can't be decoded.
/// Supports any format the `image` crate handles (PNG, JPEG, WebP, …).
fn load_texture(path: &str) -> Option<TextureData> {
    let img = image::open(path).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Some(TextureData {
        width: width as i32,
        height: height as i32,
        data: img.into_raw(),
    })
}

/// C-callable entry point exposed via GObject Introspection.
///
/// Creates a `GtkGLArea` that renders `shader` over a fullscreen quad.
/// Textures are loaded from disk and passed to the shader as `tex0`, `tex1`, …
///
/// Ownership of the returned widget is transferred to the caller (GTK will
/// manage its lifetime via ref-counting once it's added to a parent widget).
///
/// # Safety
///
/// - `shader` must be a valid null-terminated UTF-8 string.
/// - `texture_paths` must point to an array of `texture_count` valid
///   null-terminated strings, or be null if `texture_count` is 0.
#[no_mangle]
pub extern "C" fn gtk_gl_shaders_new_area_for_shader(
    shader: *const std::os::raw::c_char,
    texture_paths: *const *const std::os::raw::c_char,
    texture_count: i32,
) -> *mut gtk::ffi::GtkWidget {
    // GTK may already be initialized by the GJS runtime — that's fine,
    // init() is a no-op if called more than once.
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
