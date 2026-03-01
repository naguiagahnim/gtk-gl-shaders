use std::{collections::HashMap, ffi::c_char, path::PathBuf, ptr};

use glib::{
    GString, Value, Variant,
    ffi::{self, GType, GVariant},
    subclass::types::ObjectSubclass,
    translate::*,
    types::StaticType,
};
use gobject_sys::GValue;

use super::Uniform;

pub type ShaderArea = <super::imp::ShaderArea as ObjectSubclass>::Instance;

// this functions is called by g-ir-scanner
#[unsafe(no_mangle)]
pub extern "C" fn gtk_gl_shaders_shader_area_get_type() -> GType {
    // gtk need to be initialized or it panics
    gtk::init().expect("GTK initialization failed");

    <super::ShaderArea as StaticType>::static_type().into_glib()
}

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
        let uniforms = unsafe { Variant::from_glib_none(uniforms) };
        let mut result = HashMap::new();

        if let Some(uniforms) = uniforms.get::<HashMap<String, Variant>>() {
            for (name, value) in uniforms {
                let value = if let Some(value) = value.get::<f64>() {
                    Uniform::Float(value as f32)
                } else if let Some(value) = value.get::<Vec<f64>>() {
                    match value.len() {
                        2 => Uniform::Vec2([value[0] as f32, value[1] as f32]),
                        3 => Uniform::Vec3([value[0] as f32, value[1] as f32, value[2] as f32]),
                        4 => Uniform::Vec4([
                            value[0] as f32,
                            value[1] as f32,
                            value[2] as f32,
                            value[3] as f32,
                        ]),
                        n => {
                            println!("Uniform '{name}' has an invalid number of elements: {n}");
                            continue;
                        }
                    }
                } else if let Some(value) = value.get::<i32>() {
                    Uniform::Int(value)
                } else if let Some(value) = value.get::<Vec<i32>>() {
                    match value.len() {
                        2 => Uniform::IVec2([value[0], value[1]]),
                        3 => Uniform::IVec3([value[0], value[1], value[2]]),
                        4 => Uniform::IVec4([value[0], value[1], value[2], value[3]]),
                        n => {
                            println!("Uniform '{name}' has an invalid number of elements: {n}");
                            continue;
                        }
                    }
                } else {
                    println!("Uniform '{name}' has unknown type");
                    continue;
                };
                result.insert(name, value);
            }
        } else {
            println!("Invalid value passed to `uniforms`");
        }

        result
    };

    super::ShaderArea::new(shader, textures, uniforms).to_glib_full()
}

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gtk_gl_shaders_shader_area_set_uniform_vec2(
    this: *mut ShaderArea,
    name: *const c_char,
    a: f32,
    b: f32,
) {
    let this = unsafe { super::ShaderArea::from_glib_none(this) };
    let name = unsafe { GString::from_glib_none(name) }.as_str().to_owned();

    this.set_uniform(name, Uniform::Vec2([a, b]));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gtk_gl_shaders_shader_area_set_uniform_vec3(
    this: *mut ShaderArea,
    name: *const c_char,
    a: f32,
    b: f32,
    c: f32,
) {
    let this = unsafe { super::ShaderArea::from_glib_none(this) };
    let name = unsafe { GString::from_glib_none(name) }.as_str().to_owned();

    this.set_uniform(name, Uniform::Vec3([a, b, c]));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gtk_gl_shaders_shader_area_set_uniform_vec4(
    this: *mut ShaderArea,
    name: *const c_char,
    a: f32,
    b: f32,
    c: f32,
    d: f32,
) {
    let this = unsafe { super::ShaderArea::from_glib_none(this) };
    let name = unsafe { GString::from_glib_none(name) }.as_str().to_owned();

    this.set_uniform(name, Uniform::Vec4([a, b, c, d]));
}

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gtk_gl_shaders_shader_area_set_uniform_ivec2(
    this: *mut ShaderArea,
    name: *const c_char,
    a: i32,
    b: i32,
) {
    let this = unsafe { super::ShaderArea::from_glib_none(this) };
    let name = unsafe { GString::from_glib_none(name) }.as_str().to_owned();

    this.set_uniform(name, Uniform::IVec2([a, b]));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gtk_gl_shaders_shader_area_set_uniform_ivec3(
    this: *mut ShaderArea,
    name: *const c_char,
    a: i32,
    b: i32,
    c: i32,
) {
    let this = unsafe { super::ShaderArea::from_glib_none(this) };
    let name = unsafe { GString::from_glib_none(name) }.as_str().to_owned();

    this.set_uniform(name, Uniform::IVec3([a, b, c]));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gtk_gl_shaders_shader_area_set_uniform_ivec4(
    this: *mut ShaderArea,
    name: *const c_char,
    a: i32,
    b: i32,
    c: i32,
    d: i32,
) {
    let this = unsafe { super::ShaderArea::from_glib_none(this) };
    let name = unsafe { GString::from_glib_none(name) }.as_str().to_owned();

    this.set_uniform(name, Uniform::IVec4([a, b, c, d]));
}
