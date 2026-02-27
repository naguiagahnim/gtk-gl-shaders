/* This file mainly serves the purpose of testing if we can correctly import the lib through GJS.
DO NOT MOVE IT as it is referenced in the test.sh Bash file */

import Gtk from "gi://Gtk?version=4.0";
import GLib from "gi://GLib";
import GtkGlShaders from "gi://GtkGlShaders";

Gtk.init();

const win = new Gtk.Window({ title: "Test GLArea" });
win.set_default_size(400, 400);

const area = GtkGlShaders.new_area_for_shader(`
    in vec2 uv;
    out vec4 out_color;
    void main() {
        out_color = vec4(uv, 0.5, 1.0);
    }
`);

win.set_child(area);
win.connect("destroy", () => loop.quit());

const loop = GLib.MainLoop.new(null, false);
win.show();
loop.run();
