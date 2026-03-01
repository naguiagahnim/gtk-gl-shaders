use std::{collections::HashMap, path::PathBuf};

use glib::{Object, subclass::types::ObjectSubclassIsExt};
use gtk::glib;

mod ffi;
mod imp;

/// An uniform value that can be passed to shaders.
#[derive(Debug, Clone)]
pub enum Uniform {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Int(i32),
    IVec2([i32; 2]),
    IVec3([i32; 3]),
    IVec4([i32; 4]),
}

glib::wrapper! {
    pub struct ShaderArea(ObjectSubclass<imp::ShaderArea>)
        @extends gtk::Widget,
        @implements gtk::Buildable, gtk::ConstraintTarget;
}

impl ShaderArea {
    #[must_use]
    pub fn new(shader: String, textures: Vec<PathBuf>, uniforms: HashMap<String, Uniform>) -> Self {
        let this: Self = Object::new();
        this.imp().initialize(shader, textures, uniforms);
        this
    }

    pub fn set_uniform(&self, name: String, value: Uniform) {
        self.imp().set_uniform(name, value);
    }
}
