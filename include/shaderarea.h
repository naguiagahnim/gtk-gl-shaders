#pragma once
#include <glib-2.0/glib-object.h>
#include <gtk/gtk.h>

/**
 * gtk_gl_shaders_new_area_for_shader:
 * @shader: (not nullable): GLSL fragment shader source
 * @texture_paths: (array length=texture_count) (nullable): paths to image files
 * @texture_count: number of textures
 * Returns: (transfer full): a new GLArea
 */
GtkWidget *gtk_gl_shaders_new_area_for_shader(const char *shader,
                                              const char **texture_paths,
                                              int texture_count);
