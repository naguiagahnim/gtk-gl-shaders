#pragma once
#include <glib-2.0/glib-object.h>
#include <gtk/gtk.h>

/**
 * gtk_gl_shaders_new_area_for_shader:
 * @shader: GLSL fragment shader source
 * Returns: (transfer full): a new GLArea with the shader applied
 */
GtkWidget *gtk_gl_shaders_new_area_for_shader(const char *shader);
