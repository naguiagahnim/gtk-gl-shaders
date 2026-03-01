# glarea-gjs

A library made in Rust that exposes GTK4 `GLArea` widgets with custom GLSL shaders to
GJS/JavaScript via GObject Introspection. Built for use with
[AGS](https://github.com/Aylur/ags) / [Astal](https://github.com/Aylur/astal).

<div align="center">
  <img src="https://github.com/naguiagahnim/glarea-gjs/blob/657815bf231001892dd2b8cbd4aa7f75db676e9b/assets/demo.png" alt="demo" width="600">
  <p><i>A truly wondrous feat of technology</i></p>
</div>

## What it does

You write a GLSL fragment shader, pass it from JavaScript, and get back a
`GLArea` widget you can drop anywhere in your GTK4 app. The library handles the
OpenGL boilerplate — fullscreen quad, shader compilation, texture uploads,
cleanup.

```js
import GtkGlShaders from "gi://GtkGlShaders";

const area = GtkGlShaders.new_area_for_shader(
  `
    in vec2 uv;
    uniform sampler2D tex0;
    out vec4 out_color;

    void main() {
        out_color = texture(tex0, uv);
    }
`,
  ["/path/to/image.png"],
);

area.set_size_request(200, 200)
```

The `uv` interpolant goes from `(0, 0)` at the bottom-left to `(1, 1)` at the
top-right. Textures are accessible as `tex0`, `tex1`, etc.

## Features & Limitations

### Supported
- **GLSL Fragment Shaders** — Write custom fragment shaders in GLSL
- **Texture Loading** — Pass image paths, get them as `tex0`, `tex1`, etc.
- **GTK4 Integration** — Returns a standard `GLArea` widget that works anywhere in GTK4
- **Automatic Cleanup** — OpenGL resources are properly freed when widgets are destroyed
- **Multiple Instances** — Create as many shader widgets as you need

### Not Supported (yet?)
- **Vertex Shaders** — Only fragment shaders are supported (fullscreen quad is hardcoded)
- **Uniform Variables** — No way to pass custom uniforms (time, mouse position, etc.) from JS yet
- **Dynamic Texture Updates** — Textures are set at creation time and cannot be changed
- **Animation** — No built-in frame callback or time uniform
- **3D/Geometry** — This is strictly 2D fragment shader rendering

Pull requests to add these features are welcome!

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

This includes all necessary dependencies and drops you into a shell ready to build.

## Building

### With Nix (recommended)
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

This will compile the library and run a simple demo with a test shader, through the `test.js` file.

## Project structure
```
src/
  lib.rs
  shaderarea/
    mod.rs      # OpenGL logic — quad, shader compilation, texture upload
    ffi.rs      # C FFI wrapper exposed via GObject Introspection
include/
  shaderarea.h  # C header for g-ir-scanner
flake.nix       # Nix development environment
```

## References

This project benefited from studying the following resources:

- [GTK demo (glarea.c)](https://github.com/GNOME/gtk/blob/main/demos/gtk-demo/glarea.c) — full OpenGL rendering example  
- [gtk4-rs custom widget examples](https://github.com/gtk-rs/gtk4-rs/tree/main/examples/custom_widget) — custom widget patterns  
- [gobject-example-rs](https://github.com/sdroege/gobject-example-rs) — GObject library structure reference

## Acknowledgements

This project would not be what it is without [@Rayzeq](https://github.com/Rayzeq), who contributed major features, implementation work, and core ideas.  
Effectively a co-creator of this project.

