use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::c_void,
    path::{Path, PathBuf},
    ptr,
    sync::Once,
};

use glib::Propagation;
use gtk::{GLArea, glib, prelude::*, subclass::prelude::*};
use image::GenericImageView;
use libloading::os::unix::Library;

use super::Uniform;

static INIT_EPOXY: Once = Once::new();

/// The parts of the GL state that need to be shared
#[derive(Debug)]
struct GLState {
    /// The linked shader program
    program: u32,
    /// The vertex buffer
    ///
    /// This contains all the per-vertex data
    vao: u32,
    textures: Vec<u32>,
    /// Location & value of uniforms
    uniforms: HashMap<String, (i32, Uniform)>,
}

#[derive(Debug, Default)]
pub struct ShaderArea {
    /// The [`GLArea`] on which we will be drawing
    area: RefCell<Option<gtk::GLArea>>,
    gl_state: RefCell<Option<GLState>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ShaderArea {
    const NAME: &'static str = "GtkGlShadersShaderArea";
    type Type = super::ShaderArea;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<gtk::BinLayout>();
    }
}

impl ObjectImpl for ShaderArea {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        let area = GLArea::new();
        area.set_parent(&*obj);
        *self.area.borrow_mut() = Some(area);
    }

    fn dispose(&self) {
        if let Some(area) = self.area.borrow_mut().take() {
            area.unparent();
        }
    }
}

impl WidgetImpl for ShaderArea {}

impl ShaderArea {
    pub fn initialize(
        &self,
        shader: String,
        textures: Vec<PathBuf>,
        uniforms: HashMap<String, Uniform>,
    ) {
        INIT_EPOXY.call_once(Self::init_epoxy);

        let area = self.area.borrow();
        let area = area
            .as_ref()
            .expect("Missing GLArea, was this object properly initialized ?");

        // using weakrefs to prevent reference loop
        let this = self.downgrade();
        area.connect_realize(move |area| {
            area.make_current();
            if let Some(e) = area.error() {
                println!("Failed to initialize shader area: {e}");
                return;
            }

            // GTK can use either OpenGL or OpenGL ES depending on the platform.
            // The GLSL version header differs between the two.
            let glsl_version = if area.uses_es() {
                "#version 300 es\nprecision highp float;\n"
            } else {
                "#version 330 core\n"
            };

            let vertex_shader = format!(
                r"{glsl_version}
out vec2 uv;
void main() {{
    vec2 pos = vec2(float(gl_VertexID & 1),
                    float((gl_VertexID >> 1) & 1));
    uv          = pos;
    gl_Position = vec4(pos * 2.0 - 1.0, 0.0, 1.0);
}}
"
            );
            let fragment_shader = format!("{glsl_version}{shader}");

            unsafe {
                let program = Self::link_program(&vertex_shader, &fragment_shader);

                // Core profile requires a VAO even when no vertex attributes
                // are used.
                let mut vao = 0u32;
                epoxy::GenVertexArrays(1, &raw mut vao);
                epoxy::BindVertexArray(vao);

                epoxy::UseProgram(program);

                let mut texture_ids = Vec::with_capacity(textures.len());
                for (i, tex) in textures.iter().enumerate() {
                    let Some(id) = Self::load_texture(i as u32, tex) else {
                        continue;
                    };
                    texture_ids.push(id);

                    // Bind the texture to its sampler uniform (tex0, tex1, …).
                    let name = format!("tex{i}\0");
                    let loc = epoxy::GetUniformLocation(program, name.as_ptr().cast::<i8>());
                    if loc >= 0 {
                        epoxy::Uniform1i(loc, i as i32);
                    } else {
                        println!("Warn: texture not used in shader: {}", tex.display());
                    }
                }

                let mut uniform_map = HashMap::new();
                for (name, value) in &uniforms {
                    let name_c = format!("{name}\0");
                    let loc = epoxy::GetUniformLocation(program, name_c.as_ptr().cast::<i8>());
                    if loc >= 0 {
                        uniform_map.insert(name.clone(), (loc, value.clone()));
                    } else {
                        println!("Warn: uniform not used in shader: {name}");
                    }
                }

                if let Some(this) = this.upgrade() {
                    this.gl_state.borrow_mut().replace(GLState {
                        program,
                        vao,
                        textures: texture_ids,
                        uniforms: uniform_map,
                    });
                }
            }
        });
        let this = self.downgrade();
        area.connect_unrealize(move |area| {
            area.make_current();
            if let Some(e) = area.error() {
                println!("Failed to initialize shader area: {e}");
                return;
            }

            if let Some(state) = this.upgrade().and_then(|x| x.gl_state.borrow_mut().take()) {
                unsafe {
                    epoxy::DeleteProgram(state.program);
                    epoxy::DeleteVertexArrays(1, &raw const state.vao);
                    if !state.textures.is_empty() {
                        epoxy::DeleteTextures(state.textures.len() as i32, state.textures.as_ptr());
                    }
                }
            }
        });

        let this = self.downgrade();
        area.connect_render(move |area, _ctx| {
            if let Some(e) = area.error() {
                println!("Failed to initialize shader area: {e}");
                return Propagation::Stop;
            }

            if let Some(this) = this.upgrade()
                && let Some(state) = this.gl_state.borrow().as_ref()
            {
                unsafe {
                    epoxy::ClearColor(0.0, 0.0, 0.0, 0.0);
                    epoxy::Clear(epoxy::COLOR_BUFFER_BIT);

                    Self::apply_uniforms(state);
                    epoxy::BindVertexArray(state.vao);

                    for (i, &id) in state.textures.iter().enumerate() {
                        epoxy::ActiveTexture(epoxy::TEXTURE0 + i as u32);
                        epoxy::BindTexture(epoxy::TEXTURE_2D, id);
                    }

                    // TRIANGLE_STRIP with 4 vertices produces two triangles
                    // that together cover the entire quad.
                    epoxy::DrawArrays(epoxy::TRIANGLE_STRIP, 0, 4);

                    epoxy::Flush();
                }
            }

            Propagation::Stop
        });
    }

    pub fn set_uniform(&self, name: String, value: Uniform) {
        let area = self.area.borrow();
        let mut state = self.gl_state.borrow_mut();
        let (Some(area), Some(state)) = (area.as_ref(), state.as_mut()) else {
            println!("Couldn't set uniform because the widget isn't being rendered");
            return;
        };

        area.make_current();
        if let Some(e) = area.error() {
            println!("Failed to initialize set uniform: {e}");
            return;
        }

        // Check if uniform exists or find it
        let location = if let Some((location, _)) = state.uniforms.get(&name) {
            *location
        } else {
            let name_c = format!("{name}\0");
            unsafe { epoxy::GetUniformLocation(state.program, name_c.as_ptr().cast::<i8>()) }
        };

        if location < 0 {
            println!("Warn: uniform not used in shader: {name}");
            return;
        }

        // Update the uniform value
        state.uniforms.insert(name, (location, value));

        // based on the documentation this shouldn't be necessary, but it is
        area.queue_render();
    }

    /// Loads epoxy's OpenGL function pointers from the system libepoxy.
    ///
    /// This needs to happen before any GL call, and specifically before GTK's
    /// [`GLArea`] tries to render. When the library is loaded by GJS, GTK is already
    /// initialized but epoxy hasn't been pointed at the right symbols yet — this
    /// call fixes that.
    fn init_epoxy() {
        let library = unsafe { Library::new("libepoxy.so.0") }.expect("Can't find libepoxy.so.0");
        epoxy::load_with(|name| {
            unsafe { library.get::<_>(name.as_bytes()) }
                .map(|symbol| *symbol)
                .unwrap_or(ptr::null())
        });
    }

    unsafe fn compile_shader(src: &str, kind: u32) -> u32 {
        let shader = epoxy::CreateShader(kind);
        let ptr = src.as_ptr().cast::<i8>();
        let len = src.len() as i32;
        epoxy::ShaderSource(shader, 1, &raw const ptr, &raw const len);
        epoxy::CompileShader(shader);

        let mut ok = 0i32;
        epoxy::GetShaderiv(shader, epoxy::COMPILE_STATUS, &raw mut ok);
        if ok == 0 {
            let mut log_len = 0i32;
            epoxy::GetShaderiv(shader, epoxy::INFO_LOG_LENGTH, &raw mut log_len);
            let mut buf = vec![0u8; log_len as usize];
            epoxy::GetShaderInfoLog(
                shader,
                log_len,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast::<i8>(),
            );
            eprintln!("Shader compile error: {}", String::from_utf8_lossy(&buf));
        }
        shader
    }

    unsafe fn link_program(vertex: &str, fragment: &str) -> u32 {
        let vert = Self::compile_shader(vertex, epoxy::VERTEX_SHADER);
        let frag = Self::compile_shader(fragment, epoxy::FRAGMENT_SHADER);

        let program = epoxy::CreateProgram();
        epoxy::AttachShader(program, vert);
        epoxy::AttachShader(program, frag);
        epoxy::LinkProgram(program);

        let mut ok = 0i32;
        epoxy::GetProgramiv(program, epoxy::LINK_STATUS, &raw mut ok);
        if ok == 0 {
            let mut log_len = 0i32;
            epoxy::GetProgramiv(program, epoxy::INFO_LOG_LENGTH, &raw mut log_len);
            let mut buf = vec![0u8; log_len as usize];
            epoxy::GetProgramInfoLog(
                program,
                log_len,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast::<i8>(),
            );
            eprintln!("Program link error: {}", String::from_utf8_lossy(&buf));
        }

        epoxy::DeleteShader(vert);
        epoxy::DeleteShader(frag);
        program
    }

    unsafe fn load_texture(index: u32, path: &Path) -> Option<u32> {
        let image = match image::open(path) {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to load texture: {e}");
                return None;
            }
        };
        let (width, height) = image.dimensions();
        let data = image.to_rgba8().into_raw();

        let mut id = 0u32;
        epoxy::GenTextures(1, &raw mut id);
        epoxy::ActiveTexture(epoxy::TEXTURE0 + index);
        epoxy::BindTexture(epoxy::TEXTURE_2D, id);
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

    unsafe fn apply_uniforms(state: &GLState) {
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
