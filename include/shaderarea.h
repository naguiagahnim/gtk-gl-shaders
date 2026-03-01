#pragma once
#include <glib-2.0/glib-object.h>
#include <gtk/gtk.h>

/**
 * GtkGlShadersShaderArea:
 *
 * A widget to easily render OpenGL shaders.
 */
G_DECLARE_FINAL_TYPE(GtkGlShadersShaderArea, gtk_gl_shaders_shader_area,
                     GTK_GL_SHADERS, SHADER_AREA, GtkWidget)

/**
 * gtk_gl_shaders_shader_area_new:
 * @shader: (not nullable): GLSL fragment shader source
 * @textures: (array length=textures_count) (nullable): paths to image files
 * @textures_count: number of textures
 * Returns: (transfer full): a new GtkGlShadersShaderArea
 */
GtkGlShadersShaderArea *
gtk_gl_shaders_shader_area_new(const char *shader, const char **textures,
                               unsigned int textures_count);
