mod ffi;

use epoxy::ClearColor;
use gtk::{glib, prelude::*, GLArea};
use std::{collections::HashMap, sync::{Arc, Mutex, RwLock}};
use once_cell::sync::Lazy;

// Global state map for uniform updates
static STATE_MAP: Lazy<RwLock<HashMap<usize, Arc<Mutex<Option<GlState>>>>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Raw image data ready to be uploaded to the GPU.
///
/// Pixels must be in RGBA8 format (4 bytes per pixel, row-major).
/// `data.len()` must equal `width * height * 4`.
#[derive(Clone)]
pub struct TextureData {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

/// Uniform value that can be passed to shaders.
#[derive(Clone, Debug)]
pub enum UniformValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Int(i32),
    IVec2([i32; 2]),
    IVec3([i32; 3]),
    IVec4([i32; 4]),
}

/// Stored uniform information: location and current value.
#[allow(dead_code)]
struct UniformInfo {
    location: i32,
    value: UniformValue,
}

struct GlState {
    program: u32,
    vao: u32,
    textures: Vec<u32>,
    uniforms: HashMap<String, UniformInfo>,
}

/// Creates a GTK [`GLArea`] that runs a custom GLSL fragment shader over a
/// fullscreen quad.
///
/// The vertex shader is generated internally — it produces a quad that covers
/// the entire widget using `gl_VertexID`, so no vertex buffer is needed.
/// It exposes a single `in vec2 uv` interpolant to the fragment shader, with
/// `(0, 0)` at the bottom-left and `(1, 1)` at the top-right.
///
/// Each texture in `textures` is uploaded to the GPU on realize and bound to
/// the sampler uniforms `tex0`, `tex1`, … in the fragment shader.
///
/// Initial uniform values can be provided via the `uniforms` map.
///
/// # Example
///
/// ```glsl
/// in vec2 uv;
/// uniform sampler2D tex0;
/// uniform float time;
/// out vec4 out_color;
///
/// void main() {
///     out_color = texture(tex0, uv) * sin(time);
/// }
/// ```
pub fn new_area_for_shader(
    fragment_shader: String,
    textures: Vec<TextureData>,
    uniforms: HashMap<String, UniformValue>,
) -> GLArea {
    let area = GLArea::new();

    // State is stored both in closures and global map for external access
    let state: Arc<Mutex<Option<GlState>>> = Arc::new(Mutex::new(None));

    // ── realize ──────────────────────────────────────────────────────────────
    {
        let state = state.clone();
        area.connect_realize(move |widget| {
            let area = widget.downcast_ref::<GLArea>().unwrap();
            area.make_current();
            if area.error().is_some() {
                return;
            }

            // GTK can use either OpenGL or OpenGL ES depending on the platform.
            // The GLSL version header differs between the two.
            let glsl_version = if area.uses_es() {
                "#version 300 es\nprecision highp float;\n"
            } else {
                "#version 330 core\n"
            };

            let vert_src = format!(
                r#"{glsl_version}
out vec2 uv;
void main() {{
    vec2 pos = vec2(float(gl_VertexID & 1),
                    float((gl_VertexID >> 1) & 1));
    uv          = pos;
    gl_Position = vec4(pos * 2.0 - 1.0, 0.0, 1.0);
}}
"#
            );

            let frag_src = format!("{glsl_version}{fragment_shader}");

            unsafe {
                let program = link_program(&vert_src, &frag_src);

                // Core profile requires a VAO even when no vertex attributes
                // are used.
                let mut vao = 0u32;
                epoxy::GenVertexArrays(1, &mut vao);
                epoxy::BindVertexArray(vao);

                let mut tex_ids: Vec<u32> = Vec::with_capacity(textures.len());
                epoxy::UseProgram(program);

                for (i, tex) in textures.iter().enumerate() {
                    let mut id = 0u32;
                    epoxy::GenTextures(1, &mut id);
                    epoxy::ActiveTexture(epoxy::TEXTURE0 + i as u32);
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
                        tex.width,
                        tex.height,
                        0,
                        epoxy::RGBA,
                        epoxy::UNSIGNED_BYTE,
                        tex.data.as_ptr() as *const _,
                    );

                    // Bind the texture to its sampler uniform (tex0, tex1, …).
                    let name = format!("tex{i}\0");
                    let loc = epoxy::GetUniformLocation(program, name.as_ptr() as *const i8);
                    if loc >= 0 {
                        epoxy::Uniform1i(loc, i as i32);
                    }

                    tex_ids.push(id);
                }

                epoxy::UseProgram(0);
                
                // Build uniforms map by looking up locations
                let mut uniforms_map = HashMap::new();
                for (name, value) in &uniforms {
                    let name_c = format!("{}\0", name);
                    let loc = epoxy::GetUniformLocation(program, name_c.as_ptr() as *const i8);
                    if loc >= 0 {
                        uniforms_map.insert(name.clone(), UniformInfo { location: loc, value: value.clone() });
                    }
                }
                
                *state.lock().unwrap() = Some(GlState {
                    program,
                    vao,
                    textures: tex_ids,
                    uniforms: uniforms_map,
                });

                // Store state in global map keyed by widget pointer
                let widget_ptr = area.as_ptr() as usize;
                STATE_MAP.write().unwrap().insert(widget_ptr, state.clone());
            }
        });
    }

    // ── render ───────────────────────────────────────────────────────────────
    {
        let state = state.clone();
        area.connect_render(move |area, _ctx| {
            if area.error().is_some() {
                return glib::Propagation::Stop;
            }

            if let Some(ref gl) = *state.lock().unwrap() {
                unsafe {
                    ClearColor(0.0, 0.0, 0.0, 1.0);
                    epoxy::Clear(epoxy::COLOR_BUFFER_BIT);

                    // Apply uniforms before rendering
                    apply_uniforms(gl);

                    epoxy::BindVertexArray(gl.vao);

                    for (i, &id) in gl.textures.iter().enumerate() {
                        epoxy::ActiveTexture(epoxy::TEXTURE0 + i as u32);
                        epoxy::BindTexture(epoxy::TEXTURE_2D, id);
                    }

                    // TRIANGLE_STRIP with 4 vertices produces two triangles
                    // that together cover the entire quad.
                    epoxy::DrawArrays(epoxy::TRIANGLE_STRIP, 0, 4);

                    epoxy::Flush();
                }
            }

            glib::Propagation::Stop
        });
    }

    // ── unrealize ────────────────────────────────────────────────────────────
    {
        area.connect_unrealize(move |widget| {
            let area = widget.downcast_ref::<GLArea>().unwrap();
            area.make_current();
            if area.error().is_some() {
                return;
            }
            if let Ok(mut state_guard) = state.lock() {
                if let Some(gl) = state_guard.take() {
                    unsafe {
                        epoxy::DeleteProgram(gl.program);
                        epoxy::DeleteVertexArrays(1, &gl.vao);
                        if !gl.textures.is_empty() {
                            epoxy::DeleteTextures(gl.textures.len() as i32, gl.textures.as_ptr());
                        }
                    }
                }
            }
            // Clean up from global map
            let widget_ptr = area.as_ptr() as usize;
            STATE_MAP.write().unwrap().remove(&widget_ptr);
        });
    }

    area
}

/// Sets a uniform value on a GLArea created with [`new_area_for_shader`].
///
/// This function looks up the uniform by name in the shader program and updates
/// its value. The change will be applied on the next render call.
///
/// Returns `true` if the uniform was found and set, `false` if the uniform name
/// doesn't exist in the shader.
pub fn set_uniform(area: &GLArea, name: &str, value: UniformValue) -> bool {
    let widget_ptr = area.as_ptr() as usize;
    
    let state_map = STATE_MAP.read().unwrap();
    let state_arc = match state_map.get(&widget_ptr) {
        Some(arc) => arc.clone(),
        None => return false,
    };
    drop(state_map);
    
    let Ok(state_guard) = state_arc.lock() else { return false; };
    let state = match state_guard.as_ref() {
        Some(s) => s,
        None => return false,
    };
    
    // Check if uniform exists or find it
    let location = if let Some(info) = state.uniforms.get(name) {
        info.location
    } else {
        let name_c = format!("{}\0", name);
        unsafe { epoxy::GetUniformLocation(state.program, name_c.as_ptr() as *const i8) }
    };

    if location < 0 {
        return false;
    }

    // Update the uniform value
    drop(state_guard);
    let Ok(mut state_mut) = state_arc.lock() else { return false; };
    let state = state_mut.as_mut().unwrap();
    
    if let Some(info) = state.uniforms.get_mut(name) {
        info.value = value;
    } else {
        state.uniforms.insert(
            name.to_string(),
            UniformInfo {
                location,
                value,
            },
        );
    }

    true
}

/// Internal helper to apply all uniforms before rendering.
unsafe fn apply_uniforms(state: &GlState) {
    epoxy::UseProgram(state.program);
    for (name, info) in &state.uniforms {
        let name_c = format!("{}\0", name);
        let loc = epoxy::GetUniformLocation(state.program, name_c.as_ptr() as *const i8);
        if loc >= 0 {
            match &info.value {
                UniformValue::Float(v) => epoxy::Uniform1f(loc, *v),
                UniformValue::Vec2(v) => epoxy::Uniform2f(loc, v[0], v[1]),
                UniformValue::Vec3(v) => epoxy::Uniform3f(loc, v[0], v[1], v[2]),
                UniformValue::Vec4(v) => epoxy::Uniform4f(loc, v[0], v[1], v[2], v[3]),
                UniformValue::Int(v) => epoxy::Uniform1i(loc, *v),
                UniformValue::IVec2(v) => epoxy::Uniform2i(loc, v[0], v[1]),
                UniformValue::IVec3(v) => epoxy::Uniform3i(loc, v[0], v[1], v[2]),
                UniformValue::IVec4(v) => epoxy::Uniform4i(loc, v[0], v[1], v[2], v[3]),
            }
        }
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

unsafe fn compile_shader(src: &str, kind: u32) -> u32 {
    let shader = epoxy::CreateShader(kind);
    let ptr = src.as_ptr() as *const i8;
    let len = src.len() as i32;
    epoxy::ShaderSource(shader, 1, &ptr, &len);
    epoxy::CompileShader(shader);

    let mut ok = 0i32;
    epoxy::GetShaderiv(shader, epoxy::COMPILE_STATUS, &mut ok);
    if ok == 0 {
        let mut log_len = 0i32;
        epoxy::GetShaderiv(shader, epoxy::INFO_LOG_LENGTH, &mut log_len);
        let mut buf = vec![0u8; log_len as usize];
        epoxy::GetShaderInfoLog(
            shader,
            log_len,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut i8,
        );
        eprintln!("Shader compile error: {}", String::from_utf8_lossy(&buf));
    }
    shader
}

unsafe fn link_program(vert_src: &str, frag_src: &str) -> u32 {
    let vert = compile_shader(vert_src, epoxy::VERTEX_SHADER);
    let frag = compile_shader(frag_src, epoxy::FRAGMENT_SHADER);

    let program = epoxy::CreateProgram();
    epoxy::AttachShader(program, vert);
    epoxy::AttachShader(program, frag);
    epoxy::LinkProgram(program);

    let mut ok = 0i32;
    epoxy::GetProgramiv(program, epoxy::LINK_STATUS, &mut ok);
    if ok == 0 {
        let mut log_len = 0i32;
        epoxy::GetProgramiv(program, epoxy::INFO_LOG_LENGTH, &mut log_len);
        let mut buf = vec![0u8; log_len as usize];
        epoxy::GetProgramInfoLog(
            program,
            log_len,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut i8,
        );
        eprintln!("Program link error: {}", String::from_utf8_lossy(&buf));
    }

    epoxy::DeleteShader(vert);
    epoxy::DeleteShader(frag);
    program
}
