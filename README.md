# glarea-gjs

A library made in Rust that exposes GTK4 `GLArea` widgets with custom GLSL
shaders to GJS/JavaScript via GObject Introspection. Built for use with
[AGS](https://github.com/Aylur/ags) / [Astal](https://github.com/Aylur/astal).

<div align="center">
  <img src="https://github.com/naguiagahnim/glarea-gjs/blob/0b6d2742179135109cc0f59293ed3cce79384c70/assets/rainbow-demo.gif" alt="demo" width="600">
  <p><i>A truly wondrous feat of technology</i></p>
</div>

## What it does

You write a GLSL fragment shader, pass it from JavaScript along with textures
and uniform values, and get back a `ShaderArea` widget you can drop anywhere in
your GTK4 app. The library handles the OpenGL boilerplate — fullscreen quad,
shader compilation, texture uploads, uniform management, and cleanup.

```js
import GLib from "gi://GLib";
import Gtk from "gi://Gtk?version=4.0";
import GtkGlShaders from "gi://GtkGlShaders";

Gtk.init();

// Helper to convert JS objects to GVariant for uniforms
function makeUniforms(obj) {
  const entries = {};
  for (const [k, v] of Object.entries(obj)) {
    if (Array.isArray(v)) {
      entries[k] = new GLib.Variant("ad", v);
    } else if (typeof v === "number") {
      entries[k] = GLib.Variant.new_double(v);
    } else if (v instanceof GLib.Variant) {
      entries[k] = v;
    }
  }
  return new GLib.Variant("a{sv}", entries);
}

const area = GtkGlShaders.ShaderArea.new(
  `
    in vec2 uv;
    uniform sampler2D tex0;
    uniform float time;
    out vec4 out_color;

    void main() {
        vec4 texColor = texture(tex0, uv);
        // Cycle through colors based on time
        float hue = fract(time * 0.1);
        out_color = vec4(texColor.rgb * vec3(hue, 1.0 - hue, 0.5), texColor.a);
    }
`,
  ["/path/to/image.png"],
  makeUniforms({ time: 0.0 }),
);

area.set_size_request(200, 200);

// Update uniform every frame for animation
GLib.timeout_add(GLib.PRIORITY_DEFAULT, 16, () => {
  const time = GLib.get_monotonic_time() / 1000000.0;
  area.set_uniform_float("time", time);
  return GLib.SOURCE_CONTINUE;
});
```

The `uv` interpolant goes from `(0, 0)` at the bottom-left to `(1, 1)` at the
top-right. Textures are accessible as `tex0`, `tex1`, etc.

## Features

### Core Features

- **GLSL Fragment Shaders** — Write custom fragment shaders in GLSL (GLSL 330
  Core or 300 ES)
- **Texture Loading** — Pass image paths, get them as `tex0`, `tex1`, etc. in
  your shader
- **Uniform Variables** — Pass and update uniform values from JavaScript at
  runtime
- **GTK4 Integration** — Returns a `ShaderArea` widget (wrapping a `GLArea`)
  that works anywhere in GTK4
- **Automatic Cleanup** — OpenGL resources are properly freed when widgets are
  destroyed
- **Multiple Instances** — Create as many shader widgets as you need

### Uniform Types

The library supports the following uniform types that can be passed from
JavaScript:

| Type  | GLSL Type | JavaScript                             | C Function                            |
| ----- | --------- | -------------------------------------- | ------------------------------------- |
| Float | `float`   | `GLib.Variant.new_double(value)`       | `set_uniform_float(name, value)`      |
| Vec2  | `vec2`    | `new GLib.Variant('ad', [x, y])`       | `set_uniform_vec2(name, x, y)`        |
| Vec3  | `vec3`    | `new GLib.Variant('ad', [x, y, z])`    | `set_uniform_vec3(name, x, y, z)`     |
| Vec4  | `vec4`    | `new GLib.Variant('ad', [x, y, z, w])` | `set_uniform_vec4(name, x, y, z, w)`  |
| Int   | `int`     | `GLib.Variant.new_int32(value)`        | `set_uniform_int(name, value)`        |
| IVec2 | `ivec2`   | `new GLib.Variant('ai', [x, y])`       | `set_uniform_ivec2(name, x, y)`       |
| IVec3 | `ivec3`   | `new GLib.Variant('ai', [x, y, z])`    | `set_uniform_ivec3(name, x, y, z)`    |
| IVec4 | `ivec4`   | `new GLib.Variant('ai', [x, y, z, w])` | `set_uniform_ivec4(name, x, y, z, w)` |

#### Passing Uniforms at Creation

When creating a `ShaderArea`, pass uniforms as a `GVariant` dictionary:

```js
const area = GtkGlShaders.ShaderArea.new(
  shader,
  textures,
  makeUniforms({
    time: 0.0, // float
    resolution: [1920, 1080], // vec2
    color: [1.0, 0.5, 0.0, 1.0], // vec4
    seed: 42, // int
  }),
);
```

#### Updating Uniforms at Runtime

Use the typed setter methods to update uniform values after creation:

```js
area.set_uniform_float("time", elapsedTime);
area.set_uniform_vec2("resolution", width, height);
area.set_uniform_vec3("color", r, g, b);
area.set_uniform_vec4("color", r, g, b, a);
area.set_uniform_int("seed", 42);
area.set_uniform_ivec2("tileCount", 8, 8);
```

Each call to `set_uniform_*` automatically triggers a re-render of the widget.

### Example: Animated Shader

```js
import GLib from "gi://GLib";
import Gtk from "gi://Gtk?version=4.0";
import GtkGlShaders from "gi://GtkGlShaders";

Gtk.init();

const win = new Gtk.Window({ title: "Animated Shader" });
win.set_default_size(400, 400);

function makeUniforms(obj) {
  const entries = {};
  for (const [k, v] of Object.entries(obj)) {
    if (Array.isArray(v)) {
      entries[k] = new GLib.Variant("ad", v);
    } else if (typeof v === "number") {
      entries[k] = GLib.Variant.new_double(v);
    }
  }
  return new GLib.Variant("a{sv}", entries);
}

const shader = `
  in vec2 uv;
  uniform float time;
  out vec4 out_color;

  void main() {
    vec3 color = 0.5 + 0.5 * cos(time + uv.xyx + vec3(0, 2, 4));
    out_color = vec4(color, 1.0);
  }
`;

const area = GtkGlShaders.ShaderArea.new(
  shader,
  [],
  makeUniforms({ time: 0.0 }),
);
win.set_child(area);
win.show();

let startTime = GLib.get_monotonic_time() / 1000000.0;

GLib.timeout_add(GLib.PRIORITY_DEFAULT, 16, () => {
  const elapsed = GLib.get_monotonic_time() / 1000000.0 - startTime;
  area.set_uniform_float("time", elapsed);
  return GLib.SOURCE_CONTINUE;
});

Gtk.main();
```

## Limitations

### Not Implemented

- **Vertex Shaders** — Only fragment shaders are supported (fullscreen quad is
  hardcoded)
- **Dynamic Texture Updates** — Textures are set at creation time and cannot be
  changed
- **3D/Geometry** — This is strictly 2D fragment shader rendering

## Dependencies

### Runtime

- GTK4 with OpenGL support
- GJS (GNOME JavaScript)
- libepoxy

### Build-time

- Rust (stable toolchain)
- Meson + Ninja
- pkg-config
- GObject Introspection

### NixOS / Nix

A complete development environment is provided via `flake.nix`:

```bash
nix develop
```

This includes all necessary dependencies and drops you into a shell ready to
build.

## Building

### With Nix (Recommended)

```bash
nix develop          # Enter dev shell
meson setup build    # Configure build
meson compile -C build
```

### Without Nix

Ensure you have the dependencies installed via your package manager, then:

```bash
meson setup build
meson compile -C build
```

## Testing

A test script is provided to verify the library works:

```bash
./test.sh
```

This will compile the library and run a simple demo with a test shader through
the `test.js` file, showcasing uniform animation with a time-based color cycle
effect.

## Project Structure

```
src/
  lib.rs                          # Library entry point, GTK/OpenGL initialization
  shader_area/
    mod.rs                        # ShaderArea GObject wrapper, Uniform enum
    imp.rs                        # OpenGL implementation (shader compilation, textures, uniforms)
    ffi.rs                        # C FFI bindings for GJS introspection
include/
  shaderarea.h                    # C header for g-ir-scanner
flake.nix                         # Nix development environment
```

## References

This project benefited from studying the following resources:

- [GTK demo (glarea.c)](https://github.com/GNOME/gtk/blob/main/demos/gtk-demo/glarea.c)
  — full OpenGL rendering example
- [gtk4-rs custom widget examples](https://github.com/gtk-rs/gtk4-rs/tree/main/examples/custom_widget)
  — custom widget patterns
- [gobject-example-rs](https://github.com/sdroege/gobject-example-rs) — GObject
  library structure reference

## Acknowledgements

This project would not be what it is without
[@Rayzeq](https://github.com/Rayzeq), who contributed major features,
implementation work, and core ideas. Effectively a co-creator of this project.
