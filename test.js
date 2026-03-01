/* This file mainly serves the purpose of testing if we can correctly import the lib through GJS.
DO NOT MOVE IT as it is referenced in the test.sh Bash file */

import GLib from "gi://GLib";
import Gtk from "gi://Gtk?version=4.0";
import GtkGlShaders from "gi://GtkGlShaders";

Gtk.init();

const win = new Gtk.Window({ title: "Test GLArea" });
win.set_default_size(400, 400);

/// Convert the uniforms in a format that can be given to FFI
function makeUniforms(obj) {
  const entries = {};
  for (const [k, v] of Object.entries(obj)) {
    if (Array.isArray(v))
      entries[k] = new GLib.Variant('ad', v);
    else if (typeof v === "number")
      entries[k] = GLib.Variant.new_double(v);
    else if (v instanceof GLib.Variant)
      entries[k] = v;
    else
      console.log(`Invalid value for uniform: ${v}`);
  }
  return new GLib.Variant('a{sv}', entries);
}

const shader = `
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
let area = GtkGlShaders.ShaderArea.new(
  shader, ["./assets/img.jpg"], makeUniforms({ time: 0.0 })
);

win.set_child(area);
win.connect("close-request", () => {
  if (timeoutSource) {
    GLib.source_remove(timeoutSource);
    timeoutSource = null;
  }
  area = null;
  
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
  area.set_uniform_float("time", elapsedTime);

  // Queue a redraw
  // area.queue_render();

  // Continue animation
  return GLib.SOURCE_CONTINUE;
};

// Start animation loop (60 FPS)
timeoutSource = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 10, updateUniform);

win.show();
loop.run();
