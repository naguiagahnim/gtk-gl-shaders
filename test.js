#!/usr/bin/env gjs
/*
 * Test script for GtkGlShaders library.
 *
 * This script demonstrates the usage of the ShaderArea widget with
 * animated uniforms. It creates a window displaying a texture with
 * a rainbow color cycling effect.
 */

import GLib from "gi://GLib";
import Gtk from "gi://Gtk?version=4.0";
import GtkGlShaders from "gi://GtkGlShaders";

Gtk.init();

/**
 * Converts a JavaScript object of uniforms to a GVariant dictionary.
 *
 * @param {Object} obj - Object with uniform names as keys and values as numbers or arrays
 * @returns {GLib.Variant} A GVariant of type a{sv}
 */
function makeUniforms(obj) {
    const entries = {};
    for (const [key, value] of Object.entries(obj)) {
        if (Array.isArray(value)) {
            entries[key] = new GLib.Variant('ad', value);
        } else if (typeof value === "number") {
            entries[key] = GLib.Variant.new_double(value);
        } else if (value instanceof GLib.Variant) {
            entries[key] = value;
        } else {
            console.log(`Invalid value for uniform '${key}': ${value}`);
        }
    }
    return new GLib.Variant('a{sv}', entries);
}

// Fragment shader that blends a texture with a cycling rainbow color
const SHADER_SOURCE = `
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

// Create the main window
const window = new Gtk.Window({
    title: "Test GLArea",
    default_width: 400,
    default_height: 400
});

// Create the shader area widget
let shaderArea = GtkGlShaders.ShaderArea.new(
    SHADER_SOURCE,
    ["./assets/img.jpg"],
    makeUniforms({ time: 0.0 })
);

window.set_child(shaderArea);

// Animation state
let startTime = GLib.get_monotonic_time() / 1000000.0;
let timeoutSource = null;

/**
 * Animation callback that updates the time uniform.
 * @returns {boolean} GLib.SOURCE_CONTINUE to keep the timeout running
 */
function updateUniform() {
    const currentTime = GLib.get_monotonic_time() / 1000000.0;
    const elapsedTime = currentTime - startTime;

    // Update time uniform
    shaderArea.set_uniform_float("time", elapsedTime);

    // Continue animation
    return GLib.SOURCE_CONTINUE;
}

// Handle window close
window.connect("close-request", () => {
    // Clean up the animation timeout
    if (timeoutSource) {
        GLib.source_remove(timeoutSource);
        timeoutSource = null;
    }
    shaderArea = null;

    // Exit the main loop
    loop.quit();
    return false;
});

// Set up the animation loop (10ms interval = ~100 FPS)
timeoutSource = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 10, updateUniform);

// Create and run the main loop
const loop = GLib.MainLoop.new(null, false);

window.show();
loop.run();

