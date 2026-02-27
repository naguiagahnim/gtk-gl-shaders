#!/usr/bin/env bash
set -e

echo "==> Building..."
rm -rf build
meson setup build
meson compile -C build

echo "==> Running GJS test (application should boot any minute from now)..."
GI_TYPELIB_PATH=build:$GI_TYPELIB_PATH LD_LIBRARY_PATH=build GJS_DEBUG_OUTPUT=stderr GJS_DEBUG_TOPICS=JS gjs -m test.js 2>&1 | head -50

echo "==> Done!"
