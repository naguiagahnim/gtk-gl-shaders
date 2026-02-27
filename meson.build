project('gtk-gl-shaders', version: '0.1.0')

gnome = import('gnome')

headers = run_command('find', meson.project_source_root() / 'include', '-name', '*.h', check: true).stdout().strip().split('\n')

cargo_build = custom_target('cargo-build',
  output: 'libgtkglshaders.so',
  command: [
    'sh', '-c',
    'cargo build --release --manifest-path ' + meson.project_source_root() / 'Cargo.toml' +
    ' && cp ' + meson.project_source_root() / 'target/release/libgtkglshaders.so' + ' @OUTPUT@',
  ],
  build_by_default: true,
  console: true,
)

gir = custom_target('gir',
  input: cargo_build,
  output: 'GtkGlShaders-0.1.gir',
  command: [
    'g-ir-scanner', '-v', '--warn-all',
    '--namespace', 'GtkGlShaders',
    '--nsversion', '0.1',
    '--identifier-prefix', 'GtkGlShaders',
    '--symbol-prefix', 'gtk_gl_shaders',
    '-I' + meson.project_source_root() / 'include',
    '--library=gtkglshaders',
    '--library-path', meson.project_source_root() / 'target/release',
    '--include', 'GLib-2.0',    '--pkg', 'glib-2.0',
    '--include', 'GObject-2.0', '--pkg', 'gobject-2.0',
    '--include', 'Gtk-4.0',     '--pkg', 'gtk4',
    '--pkg-export', 'GtkGlShaders-0.1',
    '--output', '@OUTPUT@',
    headers,
  ],
)

typelib = custom_target('typelib',
  input: gir,
  output: 'GtkGlShaders-0.1.typelib',
  command: [
    'g-ir-compiler',
    '--shared-library=libgtkglshaders.so',
    '@INPUT@', '-o', '@OUTPUT@',
  ],
  install: true,
  install_dir: get_option('libdir') / 'girepository-1.0',
)
