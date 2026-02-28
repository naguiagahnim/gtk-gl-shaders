#pragma once
#include <glib-2.0/glib-object.h>
#include <gtk/gtk.h>
#include <stdbool.h>

/**
 * GtkGlShadersUniformType:
 * @GTK_GL_SHADERS_UNIFORM_FLOAT: Single float value
 * @GTK_GL_SHADERS_UNIFORM_VEC2: Two-component float vector
 * @GTK_GL_SHADERS_UNIFORM_VEC3: Three-component float vector
 * @GTK_GL_SHADERS_UNIFORM_VEC4: Four-component float vector
 * @GTK_GL_SHADERS_UNIFORM_INT: Single integer value
 * @GTK_GL_SHADERS_UNIFORM_IVEC2: Two-component integer vector
 * @GTK_GL_SHADERS_UNIFORM_IVEC3: Three-component integer vector
 * @GTK_GL_SHADERS_UNIFORM_IVEC4: Four-component integer vector
 */
typedef enum {
    GTK_GL_SHADERS_UNIFORM_FLOAT = 0,
    GTK_GL_SHADERS_UNIFORM_VEC2 = 1,
    GTK_GL_SHADERS_UNIFORM_VEC3 = 2,
    GTK_GL_SHADERS_UNIFORM_VEC4 = 3,
    GTK_GL_SHADERS_UNIFORM_INT = 4,
    GTK_GL_SHADERS_UNIFORM_IVEC2 = 5,
    GTK_GL_SHADERS_UNIFORM_IVEC3 = 6,
    GTK_GL_SHADERS_UNIFORM_IVEC4 = 7,
} GtkGlShadersUniformType;

/**
 * GtkGlShadersUniformValue:
 * @uniform_type: Type of the uniform
 * @data: Array of 4 floats (for int types, reinterpret as int)
 */
typedef struct {
    GtkGlShadersUniformType uniform_type;
    float data[4];
} GtkGlShadersUniformValue;

/**
 * gtk_gl_shaders_new_area_for_shader:
 * @shader: (not nullable): GLSL fragment shader source
 * @texture_paths: (array length=texture_count) (nullable): paths to image files
 * @texture_count: number of textures
 * Returns: (transfer full): a new GLArea
 *
 * Creates a new GLArea with a custom shader.
 * Use gtk_gl_shaders_set_uniform_float() or gtk_gl_shaders_set_uniform_vec4()
 * to set uniforms after creation.
 */
GtkWidget *gtk_gl_shaders_new_area_for_shader(
    const char *shader,
    const char **texture_paths,
    int texture_count);

/**
 * gtk_gl_shaders_new_area_for_shader_with_uniforms:
 * @shader: (not nullable): GLSL fragment shader source
 * @texture_paths: (array length=texture_count) (nullable): paths to image files
 * @texture_count: number of textures
 * @uniform_spec: (nullable): uniform specification string
 * Returns: (transfer full): a new GLArea
 *
 * Creates a new GLArea with a custom shader and initial uniforms.
 * @uniform_spec is a string in the format: "name1:type1:v1,v2,v3,v4;name2:type2:v1,v2,v3,v4;..."
 * where type is one of: f (float), v2 (vec2), v3 (vec3), v4 (vec4), i (int)
 * Example: "time:f:0.0;color:v4:1.0,1.0,1.0,1.0"
 */
GtkWidget *gtk_gl_shaders_new_area_for_shader_with_uniforms(
    const char *shader,
    const char **texture_paths,
    int texture_count,
    const char *uniform_spec);

/**
 * gtk_gl_shaders_set_uniform:
 * @area: (not nullable): GLArea created with gtk_gl_shaders_new_area_for_shader
 * @name: (not nullable): name of the uniform to set
 * @value: (not nullable): new uniform value
 * Returns: true if the uniform was found and set, false otherwise
 *
 * Deprecated: Use gtk_gl_shaders_set_uniform_float() or 
 * gtk_gl_shaders_set_uniform_vec4() instead.
 */
bool gtk_gl_shaders_set_uniform(GtkWidget *area, const char *name,
                                const GtkGlShadersUniformValue *value);

/**
 * gtk_gl_shaders_set_uniform_float:
 * @area: (not nullable): GLArea created with gtk_gl_shaders_new_area_for_shader
 * @name: (not nullable): name of the uniform to set
 * @value: new float value
 * Returns: true if the uniform was found and set, false otherwise
 */
bool gtk_gl_shaders_set_uniform_float(GtkWidget *area, const char *name,
                                      float value);

/**
 * gtk_gl_shaders_set_uniform_vec4:
 * @area: (not nullable): GLArea created with gtk_gl_shaders_new_area_for_shader
 * @name: (not nullable): name of the uniform to set
 * @x: first component
 * @y: second component
 * @z: third component
 * @w: fourth component
 * Returns: true if the uniform was found and set, false otherwise
 */
bool gtk_gl_shaders_set_uniform_vec4(GtkWidget *area, const char *name,
                                     float x, float y, float z, float w);
