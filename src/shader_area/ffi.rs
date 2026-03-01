//! C FFI bindings for `ShaderArea`.
//!
//! This module exposes C-compatible functions that are called through
//! GObject Introspection from GJS/JavaScript. These functions wrap the
//! safe Rust implementation and handle FFI boundary concerns like
//! pointer validation and string conversion.
//!
//! # Safety
//!
//! All functions in this module are `extern "C"` and must uphold FFI safety
//! guarantees. Callers must ensure pointers are valid and strings are
//! null-terminated.

use std::{collections::HashMap, ffi::c_char, path::PathBuf};

use glib::{
    GString, Variant,
    ffi::{GType, GVariant},
    subclass::types::ObjectSubclass,
    translate::{FromGlibPtrNone, IntoGlib, ToGlibPtr},
    types::StaticType,
};
use log::error;

use super::Uniform;

pub type ShaderArea = <super::imp::ShaderArea as ObjectSubclass>::Instance;

/// Returns the `GType` for `ShaderArea`.
///
/// This function is called by g-ir-scanner during introspection generation.
///
/// # Safety
///
/// This function is safe to call from C.
#[unsafe(no_mangle)]
pub extern "C" fn gtk_gl_shaders_shader_area_get_type() -> GType {
    <super::ShaderArea as StaticType>::static_type().into_glib()
}

/// Creates a new `ShaderArea` widget.
///
/// # Safety
///
/// - `shader` must be a valid null-terminated C string
/// - `textures` must be a valid array of `textures_count` null-terminated C strings (or null)
/// - `uniforms` must be a valid `GVariant` of type `a{sv}` (or null)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gtk_gl_shaders_shader_area_new(
    shader: *const c_char,
    textures: *const *const c_char,
    textures_count: u32,
    uniforms: *mut GVariant,
) -> *mut ShaderArea {
    let shader = unsafe { GString::from_glib_none(shader) }
        .as_str()
        .to_owned();

    let textures = if textures.is_null() {
        Vec::new()
    } else {
        unsafe {
            (0..textures_count)
                .map(|i| {
                    let path = *textures.add(i as usize);
                    PathBuf::from(GString::from_glib_none(path).as_str().to_owned())
                })
                .collect()
        }
    };

    let uniforms = if uniforms.is_null() {
        HashMap::new()
    } else {
        parse_uniforms(unsafe { Variant::from_glib_none(uniforms) })
    };

    super::ShaderArea::new(shader, textures, uniforms).to_glib_full()
}

/// Parses a `GVariant` dictionary into a `HashMap` of uniforms.
fn parse_uniforms(variant: Variant) -> HashMap<String, Uniform> {
    let mut result = HashMap::new();

    let Some(uniforms) = variant.get::<HashMap<String, Variant>>() else {
        error!("Invalid value passed to `uniforms` - expected a{{sv}} dictionary");
        return result;
    };

    for (name, value) in uniforms {
        let uniform = if let Some(v) = value.get::<f64>() {
            // Variant can't contain f32, so we cast from f64
            Uniform::Float(v as f32)
        } else if let Some(v) = value.get::<Vec<f64>>() {
            match v.len() {
                2 => Uniform::Vec2([v[0] as f32, v[1] as f32]),
                3 => Uniform::Vec3([v[0] as f32, v[1] as f32, v[2] as f32]),
                4 => Uniform::Vec4([v[0] as f32, v[1] as f32, v[2] as f32, v[3] as f32]),
                n => {
                    error!(
                        "Uniform '{name}' has invalid number of elements: {n} (expected 2, 3, or 4)"
                    );
                    continue;
                }
            }
        } else if let Some(v) = value.get::<i32>() {
            Uniform::Int(v)
        } else if let Some(v) = value.get::<Vec<i32>>() {
            match v.len() {
                2 => Uniform::IVec2([v[0], v[1]]),
                3 => Uniform::IVec3([v[0], v[1], v[2]]),
                4 => Uniform::IVec4([v[0], v[1], v[2], v[3]]),
                n => {
                    error!(
                        "Uniform '{name}' has invalid number of elements: {n} (expected 2, 3, or 4)"
                    );
                    continue;
                }
            }
        } else {
            error!("Uniform '{name}' has unsupported type");
            continue;
        };
        result.insert(name, uniform);
    }

    result
}

/// Macro to generate uniform setter FFI functions.
macro_rules! generate_uniform_setter {
    ($name:ident, $variant:ident, $($param:ident: $ty:ty),+) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            this: *mut ShaderArea,
            name: *const c_char,
            $($param: $ty),+
        ) {
            let this = unsafe { super::ShaderArea::from_glib_none(this) };
            let name = unsafe { GString::from_glib_none(name) }.as_str().to_owned();
            this.set_uniform(name, Uniform::$variant([$($param),+]));
        }
    };
    ($name:ident, $variant:ident, $param:ident: $ty:ty) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            this: *mut ShaderArea,
            name: *const c_char,
            $param: $ty
        ) {
            let this = unsafe { super::ShaderArea::from_glib_none(this) };
            let name = unsafe { GString::from_glib_none(name) }.as_str().to_owned();
            this.set_uniform(name, Uniform::$variant($param));
        }
    };
}

// Float setter (single value, not a vector)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gtk_gl_shaders_shader_area_set_uniform_float(
    this: *mut ShaderArea,
    name: *const c_char,
    value: f32,
) {
    let this = unsafe { super::ShaderArea::from_glib_none(this) };
    let name = unsafe { GString::from_glib_none(name) }.as_str().to_owned();
    this.set_uniform(name, Uniform::Float(value));
}

// Vector setters
generate_uniform_setter!(gtk_gl_shaders_shader_area_set_uniform_vec2, Vec2, a: f32, b: f32);
generate_uniform_setter!(gtk_gl_shaders_shader_area_set_uniform_vec3, Vec3, a: f32, b: f32, c: f32);
generate_uniform_setter!(gtk_gl_shaders_shader_area_set_uniform_vec4, Vec4, a: f32, b: f32, c: f32, d: f32);

// Int setter (single value)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gtk_gl_shaders_shader_area_set_uniform_int(
    this: *mut ShaderArea,
    name: *const c_char,
    value: i32,
) {
    let this = unsafe { super::ShaderArea::from_glib_none(this) };
    let name = unsafe { GString::from_glib_none(name) }.as_str().to_owned();
    this.set_uniform(name, Uniform::Int(value));
}

// Integer vector setters
generate_uniform_setter!(gtk_gl_shaders_shader_area_set_uniform_ivec2, IVec2, a: i32, b: i32);
generate_uniform_setter!(gtk_gl_shaders_shader_area_set_uniform_ivec3, IVec3, a: i32, b: i32, c: i32);
generate_uniform_setter!(gtk_gl_shaders_shader_area_set_uniform_ivec4, IVec4, a: i32, b: i32, c: i32, d: i32);
