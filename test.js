Gtk.init();

const win = new Gtk.Window({ title: "Test GLArea - Rainbow Animation" });
win.set_default_size(400, 400);

// Shader with a time uniform for rainbow animation
const fragmentShader = `
in vec2 uv;
uniform sampler2D tex0;
uniform float time;
out vec4 out_color;

// Convert HSV to RGB
vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    vec4 texColor = texture(tex0, uv);
    
    // Cycle through rainbow colors (HSV with full saturation and value)
    float hue = fract(time * 0.1);  // Slow color cycle
    vec3 rainbow = hsv2rgb(vec3(hue, 1.0, 1.0));
    
    // Blend texture with rainbow color
    out_color = vec4(texColor.rgb * rainbow, texColor.a);
}
`;

// Create with initial uniforms using string spec format: "name:type:v1,v2,v3,v4;..."
// const uniformSpec = "time:f:0.0;color:v4:1.0,1.0,1.0,1.0";
const uniformSpec = "time:f:0.0";

const area = GtkGlShaders.new_area_for_shader_with_uniforms(
    fragmentShader,
    ["./assets/img.jpg"],
    uniformSpec,
);

win.set_child(area);
win.connect("destroy", () => {
    if (timeoutSource) {
        GLib.source_remove(timeoutSource);
    }
    loop.quit();
});

const loop = GLib.MainLoop.new(null, false);

// Animation: update the time uniform every frame
let startTime = GLib.get_monotonic_time() / 1000000.0;
let timeoutSource = null;

const updateUniform = () => {
    const currentTime = GLib.get_monotonic_time() / 1000000.0;
    const elapsedTime = currentTime - startTime;

    // Update time uniform using the convenience function
    GtkGlShaders.set_uniform_float(area, "time", elapsedTime);

    // Queue a redraw
    area.queue_render();

    // Continue animation
    return GLib.SOURCE_CONTINUE;
};

// Start animation loop (60 FPS)
timeoutSource = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 10, updateUniform);

win.show();
loop.run();
