#!/usr/bin/env bash
set -e

mkdir -p generated

echo "==> Building Rust lib..."
cargo build

echo "==> Generating .gir..."
g-ir-scanner -v --warn-all \
  --namespace GtkGlShaders --nsversion 0.1 \
  --identifier-prefix GtkGlShaders \
  --symbol-prefix gtk_gl_shaders \
  -Iinclude \
  --library=gtkglshaders --library-path=target/debug \
  --include GLib-2.0 --pkg glib-2.0 \
  --include GObject-2.0 --pkg gobject-2.0 \
  --pkg-export GtkGlShaders-0.1 \
  --output generated/GtkGlShaders-0.1.gir \
  include/testobj.h # You'll have to manually include the files you want to generate here

echo "==> Compiling .typelib..."
g-ir-compiler generated/GtkGlShaders-0.1.gir \
  --shared-library=libgtkglshaders.so \
  -o generated/GtkGlShaders-0.1.typelib

echo "==> Running GJS test..."
GI_TYPELIB_PATH=$(pwd)/generated LD_LIBRARY_PATH=$(pwd)/target/debug gjs -m test.js

echo "==> Done!"
