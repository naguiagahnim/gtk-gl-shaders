//! OpenGL shader rendering widget for GTK4.
//!
//! This module provides the [`ShaderArea`] widget, which renders GLSL fragment
//! shaders on a fullscreen quad. It supports:
//!
//! - Custom fragment shaders in GLSL
//! - Multiple texture inputs (accessible as `tex0`, `tex1`, etc.)
//! - Uniform variables (float, vec2/3/4, int, ivec2/3/4)
//! - Automatic resource cleanup when the widget is destroyed
//!
//! # Example (from GJS)
//!
//! ```javascript
//! import GtkGlShaders from "gi://GtkGlShaders";
//!
//! const area = GtkGlShaders.ShaderArea.new(
//!   `
//!     in vec2 uv;
//!     uniform sampler2D tex0;
//!     out vec4 out_color;
//!
//!     void main() {
//!         out_color = texture(tex0, uv);
//!     }
//!   `,
//!   ["/path/to/image.png"],
//!   null  // uniforms
//! );
//! ```
//!
//! # Shader Inputs
//!
//! - `uv` - A `vec2` interpolant from `(0, 0)` at bottom-left to `(1, 1)` at top-right
//! - `tex0`, `tex1`, ... - Sampler uniforms for each loaded texture
//! - Custom uniforms - Can be set via the `uniforms` parameter or setter methods

use std::{collections::HashMap, path::PathBuf};

use glib::{Object, subclass::types::ObjectSubclassIsExt};
use gtk::glib;

mod ffi;
mod imp;

/// A uniform value that can be passed to shaders.
///
/// These types correspond to GLSL uniform types and can be set from GJS
/// using the appropriate setter methods on [`ShaderArea`].
#[derive(Debug, Clone)]
pub enum Uniform {
    /// A single float value (`float` in GLSL)
    Float(f32),
    /// A 2-component vector (`vec2` in GLSL)
    Vec2([f32; 2]),
    /// A 3-component vector (`vec3` in GLSL)
    Vec3([f32; 3]),
    /// A 4-component vector (`vec4` in GLSL)
    Vec4([f32; 4]),
    /// A single integer value (`int` in GLSL)
    Int(i32),
    /// A 2-component integer vector (`ivec2` in GLSL)
    IVec2([i32; 2]),
    /// A 3-component integer vector (`ivec3` in GLSL)
    IVec3([i32; 3]),
    /// A 4-component integer vector (`ivec4` in GLSL)
    IVec4([i32; 4]),
}

glib::wrapper! {
    /// A GTK4 widget that renders custom GLSL fragment shaders.
    ///
    /// `ShaderArea` is a GTK4 widget that embeds a `GLArea` and renders a fullscreen
    /// quad with a custom fragment shader. It supports loading textures and setting
    /// uniform values.
    ///
    /// # Example
    ///
    /// See the module-level documentation for usage examples.
    pub struct ShaderArea(ObjectSubclass<imp::ShaderArea>)
        @extends gtk::Widget,
        @implements gtk::Buildable, gtk::ConstraintTarget;
}

impl ShaderArea {
    /// Creates a new `ShaderArea` widget.
    ///
    /// # Arguments
    ///
    /// * `shader` - GLSL fragment shader source code
    /// * `textures` - Paths to image files to load as textures (accessible as `tex0`, `tex1`, etc.)
    /// * `uniforms` - Initial uniform values to pass to the shader
    ///
    /// # Returns
    ///
    /// A new `ShaderArea` widget ready to be added to a GTK4 container.
    #[must_use]
    pub fn new(shader: String, textures: Vec<PathBuf>, uniforms: HashMap<String, Uniform>) -> Self {
        let this: Self = Object::new();
        this.imp().initialize(shader, textures, uniforms);
        this
    }

    /// Sets a uniform value on the shader.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the uniform variable in the shader
    /// * `value` - The value to set
    ///
    /// # Note
    ///
    /// If the widget is not yet realized, the uniform will be set once it is.
    /// If the uniform name doesn't exist in the shader, a warning is logged.
    pub fn set_uniform(&self, name: String, value: Uniform) {
        self.imp().set_uniform(name, value);
    }
}
