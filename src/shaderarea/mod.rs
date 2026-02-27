pub mod ffi;

use gtk::GLArea;

const VERTEX_DATA: &[f32] = &[
     0.0,  0.5,   0.0, 1.0,  1.0, 0.0, 0.0, 1.0,
     0.5, -0.366, 0.0, 1.0,  0.0, 1.0, 0.0, 1.0,
    -0.5, -0.366, 0.0, 1.0,  0.0, 0.0, 1.0, 1.0,
];

pub fn new_area_for_shader(shader: String, textures: Vec<Vec<u8>>) -> GLArea {
    let area = GLArea::new();
    let fragment_shader = 
    area
}
