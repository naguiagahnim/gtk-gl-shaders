use super::{new_area_for_shader, set_uniform, UniformValue};
use crate::shaderarea::TextureData;
use glib::object::Cast;
use glib::translate::{FromGlibPtrNone, IntoGlibPtr};
use std::collections::HashMap;
use std::ffi::CStr;
use std::ptr;

/// Uniform type enum for FFI.
#[repr(i32)]
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum UniformType {
    Float = 0,
    Vec2 = 1,
    Vec3 = 2,
    Vec4 = 3,
    Int = 4,
    IVec2 = 5,
    IVec3 = 6,
    IVec4 = 7,
}

/// C-compatible uniform value.
#[repr(C)]
pub struct UniformValueFFI {
    pub uniform_type: UniformType,
    pub data: [f32; 4], // Used for both float and int (reinterpret cast)
}

impl UniformValueFFI {
    fn to_uniform_value(&self) -> UniformValue {
        match self.uniform_type {
            UniformType::Float => UniformValue::Float(self.data[0]),
            UniformType::Vec2 => UniformValue::Vec2([self.data[0], self.data[1]]),
            UniformType::Vec3 => UniformValue::Vec3([self.data[0], self.data[1], self.data[2]]),
            UniformType::Vec4 => UniformValue::Vec4([self.data[0], self.data[1], self.data[2], self.data[3]]),
            UniformType::Int => UniformValue::Int(self.data[0] as i32),
            UniformType::IVec2 => UniformValue::IVec2([self.data[0] as i32, self.data[1] as i32]),
            UniformType::IVec3 => UniformValue::IVec3([self.data[0] as i32, self.data[1] as i32, self.data[2] as i32]),
            UniformType::IVec4 => UniformValue::IVec4([self.data[0] as i32, self.data[1] as i32, self.data[2] as i32, self.data[3] as i32]),
        }
    }
}

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
    gtk_gl_shaders_new_area_for_shader_with_uniforms(shader, texture_paths, texture_count, std::ptr::null())
}

/// Parse uniform specification string and return HashMap of uniforms.
/// Format: "name1:type1:v1,v2,v3,v4;name2:type2:v1,v2,v3,v4;..."
/// Types: f (float), v2 (vec2), v3 (vec3), v4 (vec4), i (int)
fn parse_uniform_spec(spec: &str) -> HashMap<String, UniformValue> {
    let mut uniforms = HashMap::new();
    
    for uniform_str in spec.split(';') {
        let uniform_str = uniform_str.trim();
        if uniform_str.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = uniform_str.split(':').collect();
        if parts.len() != 3 {
            eprintln!("Invalid uniform spec: {}", uniform_str);
            continue;
        }
        
        let name = parts[0].to_string();
        let type_str = parts[1];
        let values_str = parts[2];
        let values: Vec<f32> = values_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        
        let value = match type_str {
            "f" => UniformValue::Float(values.get(0).copied().unwrap_or(0.0)),
            "v2" => UniformValue::Vec2([
                values.get(0).copied().unwrap_or(0.0),
                values.get(1).copied().unwrap_or(0.0),
            ]),
            "v3" => UniformValue::Vec3([
                values.get(0).copied().unwrap_or(0.0),
                values.get(1).copied().unwrap_or(0.0),
                values.get(2).copied().unwrap_or(0.0),
            ]),
            "v4" => UniformValue::Vec4([
                values.get(0).copied().unwrap_or(0.0),
                values.get(1).copied().unwrap_or(0.0),
                values.get(2).copied().unwrap_or(0.0),
                values.get(3).copied().unwrap_or(0.0),
            ]),
            "i" => UniformValue::Int(values.get(0).map(|v| *v as i32).unwrap_or(0)),
            _ => {
                eprintln!("Unknown uniform type: {}", type_str);
                continue;
            }
        };
        
        uniforms.insert(name, value);
    }
    
    uniforms
}

/// C-callable entry point with uniform support via string spec.
#[no_mangle]
pub extern "C" fn gtk_gl_shaders_new_area_for_shader_with_uniforms(
    shader: *const std::os::raw::c_char,
    texture_paths: *const *const std::os::raw::c_char,
    texture_count: i32,
    uniform_spec: *const std::os::raw::c_char,
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

    let uniforms: HashMap<String, UniformValue> = unsafe {
        if uniform_spec.is_null() {
            HashMap::new()
        } else {
            let spec = CStr::from_ptr(uniform_spec).to_string_lossy();
            parse_uniform_spec(&spec)
        }
    };

    let area = new_area_for_shader(shader_str, textures, uniforms);
    let widget: gtk::Widget = area.upcast();
    unsafe { widget.into_glib_ptr() }
}

/// Sets a uniform value on a GLArea created with `gtk_gl_shaders_new_area_for_shader`.
///
/// Returns `true` if the uniform was found and set, `false` otherwise.
///
/// # Safety
///
/// - `area` must be a valid pointer to a GLArea created by this library.
/// - `name` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub extern "C" fn gtk_gl_shaders_set_uniform(
    area: *mut gtk::ffi::GtkWidget,
    name: *const std::os::raw::c_char,
    value: *const UniformValueFFI,
) -> bool {
    if area.is_null() || name.is_null() || value.is_null() {
        return false;
    }

    let area = unsafe { gtk::Widget::from_glib_none(area) };
    let gl_area = area.downcast_ref::<gtk::GLArea>().unwrap();
    let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };
    let value = unsafe { (*value).to_uniform_value() };

    set_uniform(gl_area, &name_str, value)
}

/// Sets a float uniform on a GLArea.
///
/// Returns `true` if the uniform was found and set, `false` otherwise.
#[no_mangle]
pub extern "C" fn gtk_gl_shaders_set_uniform_float(
    area: *mut gtk::ffi::GtkWidget,
    name: *const std::os::raw::c_char,
    value: f32,
) -> bool {
    if area.is_null() || name.is_null() {
        return false;
    }

    let area = unsafe { gtk::Widget::from_glib_none(area) };
    let gl_area = area.downcast_ref::<gtk::GLArea>().unwrap();
    let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };

    set_uniform(gl_area, &name_str, UniformValue::Float(value))
}

/// Sets a vec4 uniform on a GLArea.
///
/// Returns `true` if the uniform was found and set, `false` otherwise.
#[no_mangle]
pub extern "C" fn gtk_gl_shaders_set_uniform_vec4(
    area: *mut gtk::ffi::GtkWidget,
    name: *const std::os::raw::c_char,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) -> bool {
    if area.is_null() || name.is_null() {
        return false;
    }

    let area = unsafe { gtk::Widget::from_glib_none(area) };
    let gl_area = area.downcast_ref::<gtk::GLArea>().unwrap();
    let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };

    set_uniform(gl_area, &name_str, UniformValue::Vec4([x, y, z, w]))
}
