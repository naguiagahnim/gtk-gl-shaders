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
 * @uniforms: (nullable): uniforms to pass to the shader
 * Returns: (transfer full): a new GtkGlShadersShaderArea
 */
GtkGlShadersShaderArea *
gtk_gl_shaders_shader_area_new(const char *shader, const char **textures,
                               unsigned int textures_count,
                               const GVariant *uniforms);

/**
 * gtk_gl_shaders_shader_area_set_uniform_float:
 * @this: (not nullable): area to modify
 * @name: (not nullable): name of the uniform
 * @value: value of the uniform
 */
void gtk_gl_shaders_shader_area_set_uniform_float(
    const GtkGlShadersShaderArea *this, const char *name, float value);

/**
 * gtk_gl_shaders_shader_area_set_uniform_vec2:
 * @this: (not nullable): area to modify
 * @name: (not nullable): name of the uniform
 */
void gtk_gl_shaders_shader_area_set_uniform_vec2(
    const GtkGlShadersShaderArea *this, const char *name, float a, float b);

/**
 * gtk_gl_shaders_shader_area_set_uniform_vec3:
 * @this: (not nullable): area to modify
 * @name: (not nullable): name of the uniform
 */
void gtk_gl_shaders_shader_area_set_uniform_vec3(
    const GtkGlShadersShaderArea *this, const char *name, float a, float b,
    float c);

/**
 * gtk_gl_shaders_shader_area_set_uniform_vec4:
 * @this: (not nullable): area to modify
 * @name: (not nullable): name of the uniform
 */
void gtk_gl_shaders_shader_area_set_uniform_vec4(
    const GtkGlShadersShaderArea *this, const char *name, float a, float b,
    float c, float d);

/**
 * gtk_gl_shaders_shader_area_set_uniform_int:
 * @this: (not nullable): area to modify
 * @name: (not nullable): name of the uniform
 * @value: value of the uniform
 */
void gtk_gl_shaders_shader_area_set_uniform_int(
    const GtkGlShadersShaderArea *this, const char *name, int value);

/**
 * gtk_gl_shaders_shader_area_set_uniform_ivec2:
 * @this: (not nullable): area to modify
 * @name: (not nullable): name of the uniform
 */
void gtk_gl_shaders_shader_area_set_uniform_ivec2(
    const GtkGlShadersShaderArea *this, const char *name, int a, int b);

/**
 * gtk_gl_shaders_shader_area_set_uniform_ivec3:
 * @this: (not nullable): area to modify
 * @name: (not nullable): name of the uniform
 */
void gtk_gl_shaders_shader_area_set_uniform_ivec3(
    const GtkGlShadersShaderArea *this, const char *name, int a, int b, int c);

/**
 * gtk_gl_shaders_shader_area_set_uniform_ivec4:
 * @this: (not nullable): area to modify
 * @name: (not nullable): name of the uniform
 */
void gtk_gl_shaders_shader_area_set_uniform_ivec4(
    const GtkGlShadersShaderArea *this, const char *name, int a, int b, int c,
    int d);