use std::{ffi::c_char, path::PathBuf};

use glib::{GString, ffi::GType, subclass::types::ObjectSubclass, translate::*, types::StaticType};

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
) -> *mut ShaderArea {
    let shader = unsafe { GString::from_glib_borrow(shader).as_str().to_owned() };

    let textures = if textures.is_null() {
        Vec::new()
    } else {
        unsafe {
            (0..textures_count)
                .map(|i| {
                    let path = *textures.add(i as usize);
                    PathBuf::from(GString::from_glib_borrow(path).as_str().to_owned())
                })
                .collect()
        }
    };

    super::ShaderArea::new(shader, textures).to_glib_full()
}
