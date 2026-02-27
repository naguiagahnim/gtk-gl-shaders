#!/usr/bin/env bash
set -e

echo "==> Building..."
meson compile -C build

echo "==> Running GJS test..."
GI_TYPELIB_PATH=build:$GI_TYPELIB_PATH LD_LIBRARY_PATH=build GJS_DEBUG_OUTPUT=stderr GJS_DEBUG_TOPICS=JS gjs -m test.js 2>&1 | head -50

echo "==> Done!"
