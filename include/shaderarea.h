#pragma once

#include <glib-2.0/glib-object.h>
#include <gtk/gtk.h>

G_BEGIN_DECLS

/**
 * GtkGlShadersShaderArea:
 *
 * A widget that renders custom GLSL fragment shaders.
 *
 * `GtkGlShadersShaderArea` is a GTK4 widget that embeds an OpenGL rendering area
 * and executes a user-provided fragment shader on a fullscreen quad. It supports
 * loading textures and setting uniform values.
 *
 * # Shader inputs
 *
 * - `uv` - A `vec2` from (0,0) at bottom-left to (1,1) at top-right
 * - `tex0`, `tex1`, ... - Sampler uniforms for loaded textures
 * - Custom uniforms - Set via the `gtk_gl_shaders_shader_area_set_uniform_*` functions
 */
G_DECLARE_FINAL_TYPE(GtkGlShadersShaderArea, gtk_gl_shaders_shader_area,
                     GTK_GL_SHADERS, SHADER_AREA, GtkWidget)

/**
 * gtk_gl_shaders_shader_area_new:
 * @shader: (not nullable): GLSL fragment shader source code
 * @textures: (array length=textures_count) (nullable): paths to image files to load as textures
 * @textures_count: number of texture paths in the @textures array
 * @uniforms: (nullable): initial uniform values as a `GVariant` dictionary
 * Returns: (transfer full) (not nullable): a new `GtkGlShadersShaderArea` widget
 *
 * Creates a new shader widget with the given fragment shader, textures, and uniforms.
 *
 * The fragment shader should declare `in vec2 uv` for texture coordinates and
 * `out vec4 out_color` for the output color. Textures are accessible as
 * `uniform sampler2D tex0`, `tex1`, etc.
 *
 * Uniforms should be passed as a `GVariant` of type `a{sv}` (dictionary of
 * string to variant). Float values use type 'd' (double), and vector values
 * use type 'ad' (array of doubles).
 */
GtkGlShadersShaderArea *
gtk_gl_shaders_shader_area_new(const char *shader, const char **textures,
                               unsigned int textures_count,
                               const GVariant *uniforms);

/**
 * gtk_gl_shaders_shader_area_set_uniform_float:
 * @this: (not nullable): the shader area widget
 * @name: (not nullable): name of the uniform to set
 * @value: value to set
 *
 * Sets a `float` uniform value.
 */
void gtk_gl_shaders_shader_area_set_uniform_float(
    GtkGlShadersShaderArea *this, const char *name, float value);

/**
 * gtk_gl_shaders_shader_area_set_uniform_vec2:
 * @this: (not nullable): the shader area widget
 * @name: (not nullable): name of the uniform to set
 * @a: first component
 * @b: second component
 *
 * Sets a `vec2` uniform value.
 */
void gtk_gl_shaders_shader_area_set_uniform_vec2(
    GtkGlShadersShaderArea *this, const char *name, float a, float b);

/**
 * gtk_gl_shaders_shader_area_set_uniform_vec3:
 * @this: (not nullable): the shader area widget
 * @name: (not nullable): name of the uniform to set
 * @a: first component
 * @b: second component
 * @c: third component
 *
 * Sets a `vec3` uniform value.
 */
void gtk_gl_shaders_shader_area_set_uniform_vec3(
    GtkGlShadersShaderArea *this, const char *name, float a, float b,
    float c);

/**
 * gtk_gl_shaders_shader_area_set_uniform_vec4:
 * @this: (not nullable): the shader area widget
 * @name: (not nullable): name of the uniform to set
 * @a: first component
 * @b: second component
 * @c: third component
 * @d: fourth component
 *
 * Sets a `vec4` uniform value.
 */
void gtk_gl_shaders_shader_area_set_uniform_vec4(
    GtkGlShadersShaderArea *this, const char *name, float a, float b,
    float c, float d);

/**
 * gtk_gl_shaders_shader_area_set_uniform_int:
 * @this: (not nullable): the shader area widget
 * @name: (not nullable): name of the uniform to set
 * @value: value to set
 *
 * Sets an `int` uniform value.
 */
void gtk_gl_shaders_shader_area_set_uniform_int(
    GtkGlShadersShaderArea *this, const char *name, int value);

/**
 * gtk_gl_shaders_shader_area_set_uniform_ivec2:
 * @this: (not nullable): the shader area widget
 * @name: (not nullable): name of the uniform to set
 * @a: first component
 * @b: second component
 *
 * Sets an `ivec2` uniform value.
 */
void gtk_gl_shaders_shader_area_set_uniform_ivec2(
    GtkGlShadersShaderArea *this, const char *name, int a, int b);

/**
 * gtk_gl_shaders_shader_area_set_uniform_ivec3:
 * @this: (not nullable): the shader area widget
 * @name: (not nullable): name of the uniform to set
 * @a: first component
 * @b: second component
 * @c: third component
 *
 * Sets an `ivec3` uniform value.
 */
void gtk_gl_shaders_shader_area_set_uniform_ivec3(
    GtkGlShadersShaderArea *this, const char *name, int a, int b, int c);

/**
 * gtk_gl_shaders_shader_area_set_uniform_ivec4:
 * @this: (not nullable): the shader area widget
 * @name: (not nullable): name of the uniform to set
 * @a: first component
 * @b: second component
 * @c: third component
 * @d: fourth component
 *
 * Sets an `ivec4` uniform value.
 */
void gtk_gl_shaders_shader_area_set_uniform_ivec4(
    GtkGlShadersShaderArea *this, const char *name, int a, int b, int c,
    int d);

G_END_DECLS