mod ffi;

use epoxy::ClearColor;
use gtk::{glib, prelude::*, GLArea};
use std::{cell::RefCell, rc::Rc};

/// Texture to upload to the GPU. Fields are FFI-safe.
#[derive(Clone)]
pub struct TextureData {
    pub width: i32,
    pub height: i32,
    /// Raw RGBA8 bytes, length must equal width * height * 4.
    pub data: Vec<u8>,
}

struct GlState {
    program: u32,
    vao: u32,
    textures: Vec<u32>,
}

/// Creates a [`GLArea`] that covers its entire surface with a single quad and
/// runs `fragment_shader` on every fragment.
///
/// The vertex shader exposes no attributes; positions are derived from
/// `gl_VertexID` so **no vertex buffer is needed**.
///
/// Each element of `textures` is uploaded as a `sampler2D` uniform named
/// `tex0`, `tex1`, … in the fragment shader.
pub fn new_area_for_shader(fragment_shader: String, textures: Vec<TextureData>) -> GLArea {
    let area = GLArea::new();

    // Shared GL state, initialized on realize, dropped on unrealize.
    let state: Rc<RefCell<Option<GlState>>> = Rc::new(RefCell::new(None));

    // ── realize ──────────────────────────────────────────────────────────────
    {
        let state = state.clone();
        area.connect_realize(move |widget| {
            let area = widget.downcast_ref::<GLArea>().unwrap();
            area.make_current();
            if area.error().is_some() {
                return;
            }

            // Choose GLSL version based on the API in use.
            // let glsl_version = if area.api() == gtk::gdk::GLAPI::GLES {
            //     "#version 300 es\nprecision highp float;\n"
            // } else {
            //     "#version 330 core\n"
            // };
            let glsl_version = "#version 330 core\n";

            // ------------------------------------------------------------------
            // Vertex shader – generates a full-screen quad from gl_VertexID,
            // no VBO required.
            // Winding:  ID 0 → BL, 1 → BR, 2 → TL, 3 → TR  (TRIANGLE_STRIP)
            // uv ∈ [0,1]² is passed to the fragment shader.
            // ------------------------------------------------------------------
            let vert_src = format!(
                r#"{glsl_version}
out vec2 uv;
void main() {{
    vec2 pos = vec2(float(gl_VertexID & 1),
                    float((gl_VertexID >> 1) & 1));
    uv          = pos;                          // [0,1]²
    gl_Position = vec4(pos * 2.0 - 1.0, 0.0, 1.0);
}}
"#
            );

            let frag_src = format!("{glsl_version}{fragment_shader}");

            unsafe {
                let program = link_program(&vert_src, &frag_src);

                // A VAO is mandatory in core profiles even without any buffers.
                let mut vao = 0u32;
                epoxy::GenVertexArrays(1, &mut vao);
                epoxy::BindVertexArray(vao);

                // Upload every texture and bind it to the matching sampler.
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

                    // Bind sampler uniform: tex0, tex1, …
                    let name = format!("tex{i}\0");
                    let loc = epoxy::GetUniformLocation(program, name.as_ptr() as *const i8);
                    if loc >= 0 {
                        epoxy::Uniform1i(loc, i as i32);
                    }

                    tex_ids.push(id);
                }

                epoxy::UseProgram(0);
                *state.borrow_mut() = Some(GlState {
                    program,
                    vao,
                    textures: tex_ids,
                });
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

            if let Some(ref gl) = *state.borrow() {
                unsafe {
                    ClearColor(0.0, 0.0, 0.0, 1.0);
                    epoxy::Clear(epoxy::COLOR_BUFFER_BIT);

                    epoxy::UseProgram(gl.program);
                    epoxy::BindVertexArray(gl.vao);

                    for (i, &id) in gl.textures.iter().enumerate() {
                        epoxy::ActiveTexture(epoxy::TEXTURE0 + i as u32);
                        epoxy::BindTexture(epoxy::TEXTURE_2D, id);
                    }

                    // 4 vertices, no index buffer → two triangles as a strip.
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
            if let Some(gl) = state.borrow_mut().take() {
                unsafe {
                    epoxy::DeleteProgram(gl.program);
                    epoxy::DeleteVertexArrays(1, &gl.vao);
                    if !gl.textures.is_empty() {
                        epoxy::DeleteTextures(gl.textures.len() as i32, gl.textures.as_ptr());
                    }
                }
            }
        });
    }

    area
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
