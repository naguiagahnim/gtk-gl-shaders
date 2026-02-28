/* This file mainly serves the purpose of testing if we can correctly import the lib through GJS.
DO NOT MOVE IT as it is referenced in the test.sh Bash file */

import Gtk from "gi://Gtk?version=4.0";
import GLib from "gi://GLib";
import GtkGlShaders from "gi://GtkGlShaders";

Gtk.init();

const win = new Gtk.Window({ title: "Test GLArea - Uniform Animation Test" });
win.set_default_size(400, 400);

// Shader with a time uniform for animation
const fragmentShader = `
in vec2 uv;
uniform sampler2D tex0;
uniform float time;
uniform vec4 color;
out vec4 out_color;

void main() {
    vec4 texColor = texture(tex0, uv);
    // Animate color using time: pulse effect
    float pulse = (sin(time * 2.0) + 1.0) * 0.5;
    out_color = texColor * color * (0.5 + 0.5 * pulse);
}
`;

// Create with initial uniforms using string spec format: "name:type:v1,v2,v3,v4;..."
const uniformSpec = "time:f:0.0;color:v4:1.0,1.0,1.0,1.0";

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
timeoutSource = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 16, updateUniform);

win.show();
loop.run();
