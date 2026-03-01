use std::path::PathBuf;

use glib::{subclass::types::ObjectSubclassIsExt, Object};
use gtk::glib;

mod ffi;
mod imp;

glib::wrapper! {
    pub struct ShaderArea(ObjectSubclass<imp::ShaderArea>)
        @extends gtk::Widget,
        @implements gtk::Buildable, gtk::ConstraintTarget;
}

impl ShaderArea {
    #[must_use]
    pub fn new(shader: String, textures: Vec<PathBuf>) -> Self {
        let this: Self = Object::new();
        this.imp().initialize(shader, textures);
        this
    }
}
