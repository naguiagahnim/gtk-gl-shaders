/* This file mainly serves the purpose of testing if we can correctly import the lib through GJS.
DO NOT MOVE IT as it is referenced in the test.sh Bash file */

import GLib from "gi://GLib";
import Gtk from "gi://Gtk?version=4.0";
import GtkGlShaders from "gi://GtkGlShaders";

Gtk.init();

const win = new Gtk.Window({ title: "Test GLArea" });
win.set_default_size(400, 400);

const area = GtkGlShaders.ShaderArea.new(
  `
in vec2 uv;
    uniform sampler2D tex0;
    out vec4 out_color;
    void main() {
        out_color = texture(tex0, uv);
    }
`,
  ["./assets/img.jpg"],
);

win.set_child(area);
win.connect("destroy", () => loop.quit());

const loop = GLib.MainLoop.new(null, false);
win.show();
loop.run();
