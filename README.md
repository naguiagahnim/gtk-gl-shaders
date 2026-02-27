# glarea-gjs

A Rust library that exposes GTK4 `GLArea` widgets with custom GLSL shaders to
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
```

The `uv` interpolant goes from `(0, 0)` at the bottom-left to `(1, 1)` at the
top-right. Textures are accessible as `tex0`, `tex1`, etc.

## Requirements

- NixOS or any Linux with GTK4, GJS, GObject Introspection, libepoxy
- Rust (stable)
- Meson + Ninja

## Building

```bash
nix develop
meson setup build
meson compile -C build
```

## Running the test

```bash
./test.sh
```

## Project structure

```
src/
  lib.rs
  shaderarea/
    mod.rs      # OpenGL logic — quad, shader compilation, texture upload
    ffi.rs      # C FFI wrapper exposed via GObject Introspection
include/
  shaderarea.h  # C header for g-ir-scanner
```

## Acknowledgements

Special thanks to [@Rayzeq](https://github.com/Rayzeq) for figuring out the
epoxy initialization issue and for the help along the way
