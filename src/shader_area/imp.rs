//! Internal implementation of the `ShaderArea` widget.
//!
//! This module contains the GTK4 subclass implementation and all OpenGL
//! rendering logic including shader compilation, texture loading, and
//! uniform management.

use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::c_void,
    path::{Path, PathBuf},
};

use glib::Propagation;
use gtk::{glib, prelude::*, subclass::prelude::*};
use image::GenericImageView;
use log::{error, warn};

use super::Uniform;

/// OpenGL state shared across rendering callbacks.
#[derive(Debug)]
struct GLState {
    /// Linked shader program
    program: u32,
    /// Vertex array object for the fullscreen quad
    vao: u32,
    /// Loaded texture IDs
    textures: Vec<u32>,
    /// Uniform locations and values: name -> (location, value)
    uniforms: HashMap<String, (i32, Uniform)>,
}

/// Internal state for the `ShaderArea` widget.
#[derive(Debug, Default)]
pub struct ShaderArea {
    /// OpenGL state (initialized on realize, cleaned up on unrealize)
    gl_state: RefCell<Option<GLState>>,
    /// Shader source code (set before realize)
    shader_source: RefCell<Option<String>>,
    /// Texture paths (set before realize)
    texture_paths: RefCell<Option<Vec<PathBuf>>>,
    /// Initial uniforms (set before realize)
    initial_uniforms: RefCell<Option<HashMap<String, Uniform>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ShaderArea {
    const NAME: &'static str = "GtkGlShadersShaderArea";
    type Type = super::ShaderArea;
    type ParentType = gtk::GLArea;
}

impl ObjectImpl for ShaderArea {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for ShaderArea {
    fn realize(&self) {
        self.parent_realize();

        self.obj().make_current();
        if let Some(e) = self.obj().error() {
            error!("Failed to switch OpenGL context: {e}");
            return;
        }

        // Get initialization data
        let Some(shader) = self.shader_source.borrow_mut().take() else {
            error!("Shader source not set");
            return;
        };
        let Some(textures) = self.texture_paths.borrow_mut().take() else {
            error!("Texture paths not set");
            return;
        };
        let Some(uniforms) = self.initial_uniforms.borrow_mut().take() else {
            error!("Uniforms not set");
            return;
        };

        // GTK can use either OpenGL or OpenGL ES depending on the platform.
        // The GLSL version header differs between the two.
        let glsl_version = if self.obj().uses_es() {
            "#version 300 es\nprecision highp float;\n"
        } else {
            "#version 330 core\n"
        };

        let vertex_shader = Self::build_vertex_shader(glsl_version);
        let fragment_shader = format!("{glsl_version}{shader}");

        unsafe {
            let program = Self::link_program(&vertex_shader, &fragment_shader);

            // Core profile requires a VAO even when no vertex attributes are used
            let mut vao = 0u32;
            epoxy::GenVertexArrays(1, &raw mut vao);
            epoxy::BindVertexArray(vao);

            epoxy::UseProgram(program);

            // Load textures and bind them to texture units
            let mut texture_ids = Vec::with_capacity(textures.len());
            for (i, tex) in textures.iter().enumerate() {
                let Some(id) = Self::load_texture(i as u32, tex) else {
                    continue;
                };
                texture_ids.push(id);

                // Bind the texture to its sampler uniform (tex0, tex1, …)
                let name = format!("tex{i}\0");
                let loc = epoxy::GetUniformLocation(program, name.as_ptr().cast::<i8>());
                if loc >= 0 {
                    epoxy::Uniform1i(loc, i as i32);
                } else {
                    warn!("Texture not used in shader: {}", tex.display());
                }
            }

            // Collect uniform locations
            let mut uniform_map = HashMap::new();
            for (name, value) in &uniforms {
                let name_c = format!("{name}\0");
                let loc = epoxy::GetUniformLocation(program, name_c.as_ptr().cast::<i8>());
                if loc >= 0 {
                    uniform_map.insert(name.clone(), (loc, value.clone()));
                } else {
                    warn!("Uniform not used in shader: {name}");
                }
            }

            self.gl_state.borrow_mut().replace(GLState {
                program,
                vao,
                textures: texture_ids,
                uniforms: uniform_map,
            });
        }
    }

    fn unrealize(&self) {
        self.obj().make_current();
        if let Some(e) = self.obj().error() {
            error!("Failed to switch OpenGL context: {e}");
            return;
        }

        if let Some(state) = self.gl_state.borrow_mut().take() {
            unsafe {
                epoxy::DeleteProgram(state.program);
                epoxy::DeleteVertexArrays(1, &raw const state.vao);
                if !state.textures.is_empty() {
                    epoxy::DeleteTextures(state.textures.len() as i32, state.textures.as_ptr());
                }
            }
        }

        self.parent_unrealize();
    }
}

impl GLAreaImpl for ShaderArea {
    fn render(&self, _ctx: &gtk::gdk::GLContext) -> Propagation {
        if let Some(e) = self.obj().error() {
            error!("Failed to switch OpenGL context: {e}");
            return Propagation::Stop;
        }

        if let Some(state) = self.gl_state.borrow().as_ref() {
            unsafe {
                epoxy::ClearColor(0.0, 0.0, 0.0, 0.0);
                epoxy::Clear(epoxy::COLOR_BUFFER_BIT);

                Self::apply_uniforms(state);
                epoxy::BindVertexArray(state.vao);

                // Bind textures to their respective texture units
                for (i, &id) in state.textures.iter().enumerate() {
                    epoxy::ActiveTexture(epoxy::TEXTURE0 + i as u32);
                    epoxy::BindTexture(epoxy::TEXTURE_2D, id);
                }

                // Draw a fullscreen quad using TRIANGLE_STRIP
                // 4 vertices: (0,0), (1,0), (0,1), (1,1)
                epoxy::DrawArrays(epoxy::TRIANGLE_STRIP, 0, 4);

                epoxy::Flush();
            }
        }

        Propagation::Stop
    }
}

impl ShaderArea {
    /// Stores initialization data for later use when the widget is realized.
    ///
    /// # Arguments
    ///
    /// * `shader` - GLSL fragment shader source code
    /// * `textures` - Paths to image files to load as textures
    /// * `uniforms` - Initial uniform values
    pub fn initialize(
        &self,
        shader: String,
        textures: Vec<PathBuf>,
        uniforms: HashMap<String, Uniform>,
    ) {
        *self.shader_source.borrow_mut() = Some(shader);
        *self.texture_paths.borrow_mut() = Some(textures);
        *self.initial_uniforms.borrow_mut() = Some(uniforms);
    }

    /// Sets a uniform value on the shader program.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the uniform variable
    /// * `value` - The new value to set
    pub fn set_uniform(&self, name: String, value: Uniform) {
        let mut state = self.gl_state.borrow_mut();
        let Some(state) = state.as_mut() else {
            warn!("Couldn't set uniform because the widget isn't being rendered");
            return;
        };

        self.obj().make_current();
        if let Some(e) = self.obj().error() {
            error!("Failed to switch OpenGL context: {e}");
            return;
        }

        // Get or find the uniform location
        let location = if let Some((location, _)) = state.uniforms.get(&name) {
            *location
        } else {
            let name_c = format!("{name}\0");
            unsafe { epoxy::GetUniformLocation(state.program, name_c.as_ptr().cast::<i8>()) }
        };

        if location < 0 {
            warn!("Uniform not used in shader: {name}");
            return;
        }

        // Update the uniform value
        state.uniforms.insert(name, (location, value));

        // Queue a redraw to apply the new uniform
        self.obj().queue_render();
    }

    /// Builds the vertex shader for a fullscreen quad.
    fn build_vertex_shader(glsl_version: &str) -> String {
        format!(
            r"{glsl_version}
out vec2 uv;
void main() {{
    vec2 pos = vec2(float(gl_VertexID & 1),
                    float((gl_VertexID >> 1) & 1));
    uv          = pos;
    gl_Position = vec4(pos * 2.0 - 1.0, 0.0, 1.0);
}}
"
        )
    }

    /// Compiles a shader from source.
    ///
    /// # Safety
    ///
    /// This function calls unsafe OpenGL functions. An active OpenGL context
    /// must be bound before calling.
    ///
    /// # Returns
    ///
    /// The shader ID, or 0 if compilation failed (shader is not deleted on failure)
    unsafe fn compile_shader(src: &str, kind: u32) -> u32 {
        unsafe {
            let shader = epoxy::CreateShader(kind);
            let ptr = src.as_ptr().cast::<i8>();
            let len = src.len() as i32;
            epoxy::ShaderSource(shader, 1, &raw const ptr, &raw const len);
            epoxy::CompileShader(shader);

            let mut ok = 0i32;
            epoxy::GetShaderiv(shader, epoxy::COMPILE_STATUS, &raw mut ok);
            if ok == 0 {
                Self::log_shader_error(shader, "Shader");
            }
            shader
        }
    }

    /// Links a vertex and fragment shader into a program.
    ///
    /// # Safety
    ///
    /// This function calls unsafe OpenGL functions. An active OpenGL context
    /// must be bound before calling.
    unsafe fn link_program(vertex: &str, fragment: &str) -> u32 {
        unsafe {
            let vert = Self::compile_shader(vertex, epoxy::VERTEX_SHADER);
            let frag = Self::compile_shader(fragment, epoxy::FRAGMENT_SHADER);

            let program = epoxy::CreateProgram();
            epoxy::AttachShader(program, vert);
            epoxy::AttachShader(program, frag);
            epoxy::LinkProgram(program);

            let mut ok = 0i32;
            epoxy::GetProgramiv(program, epoxy::LINK_STATUS, &raw mut ok);
            if ok == 0 {
                Self::log_shader_error(program, "Program");
            }

            epoxy::DeleteShader(vert);
            epoxy::DeleteShader(frag);
            program
        }
    }

    /// Logs shader or program compilation/linking errors.
    unsafe fn log_shader_error(id: u32, kind: &str) {
        unsafe {
            let mut log_len = 0i32;
            let info_log_fn = match kind {
                "Program" => epoxy::GetProgramInfoLog,
                _ => epoxy::GetShaderInfoLog,
            };
            info_log_fn(id, log_len, std::ptr::null_mut(), std::ptr::null_mut());
            epoxy::GetProgramiv(id, epoxy::INFO_LOG_LENGTH, &raw mut log_len);

            let mut buf = vec![0u8; log_len as usize];
            info_log_fn(
                id,
                log_len,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast::<i8>(),
            );
            error!(
                "{kind} compile/link error: {}",
                String::from_utf8_lossy(&buf)
            );
        }
    }

    /// Loads a texture from an image file.
    ///
    /// # Safety
    ///
    /// This function calls unsafe OpenGL functions. An active OpenGL context
    /// must be bound before calling.
    ///
    /// # Arguments
    ///
    /// * `index` - The texture unit index (0 for tex0, 1 for tex1, etc.)
    /// * `path` - Path to the image file
    ///
    /// # Returns
    ///
    /// The texture ID, or `None` if loading failed
    unsafe fn load_texture(index: u32, path: &Path) -> Option<u32> {
        let image = match image::open(path) {
            Ok(x) => x,
            Err(e) => {
                error!("Failed to load texture: {e}");
                return None;
            }
        };
        let (width, height) = image.dimensions();
        let data = image.to_rgba8().into_raw();

        unsafe {
            let mut id = 0u32;
            epoxy::GenTextures(1, &raw mut id);
            epoxy::ActiveTexture(epoxy::TEXTURE0 + index);
            epoxy::BindTexture(epoxy::TEXTURE_2D, id);

            // Set texture parameters
            epoxy::TexParameteri(
                epoxy::TEXTURE_2D,
                epoxy::TEXTURE_MIN_FILTER,
                epoxy::LINEAR as i32,
            );
            epoxy::TexParameteri(
                epoxy::TEXTURE_2D,
                epoxy::TEXTURE_MAG_FILTER,
                epoxy::LINEAR as i32,
            );
            epoxy::TexParameteri(
                epoxy::TEXTURE_2D,
                epoxy::TEXTURE_WRAP_S,
                epoxy::CLAMP_TO_EDGE as i32,
            );
            epoxy::TexParameteri(
                epoxy::TEXTURE_2D,
                epoxy::TEXTURE_WRAP_T,
                epoxy::CLAMP_TO_EDGE as i32,
            );

            // Upload texture data
            epoxy::TexImage2D(
                epoxy::TEXTURE_2D,
                0,
                epoxy::RGBA as i32,
                width as i32,
                height as i32,
                0,
                epoxy::RGBA,
                epoxy::UNSIGNED_BYTE,
                data.as_ptr().cast::<c_void>(),
            );

            Some(id)
        }
    }

    /// Applies all uniform values to the shader program.
    ///
    /// # Safety
    ///
    /// This function calls unsafe OpenGL functions. An active OpenGL context
    /// must be bound before calling.
    unsafe fn apply_uniforms(state: &GLState) {
        unsafe {
            epoxy::UseProgram(state.program);

            for (location, value) in state.uniforms.values() {
                match &value {
                    Uniform::Float(v) => epoxy::Uniform1f(*location, *v),
                    Uniform::Vec2(v) => epoxy::Uniform2f(*location, v[0], v[1]),
                    Uniform::Vec3(v) => epoxy::Uniform3f(*location, v[0], v[1], v[2]),
                    Uniform::Vec4(v) => epoxy::Uniform4f(*location, v[0], v[1], v[2], v[3]),
                    Uniform::Int(v) => epoxy::Uniform1i(*location, *v),
                    Uniform::IVec2(v) => epoxy::Uniform2i(*location, v[0], v[1]),
                    Uniform::IVec3(v) => epoxy::Uniform3i(*location, v[0], v[1], v[2]),
                    Uniform::IVec4(v) => epoxy::Uniform4i(*location, v[0], v[1], v[2], v[3]),
                }
            }
        }
    }
}
