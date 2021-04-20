#version 450
#extension GL_ARB_separate_shader_objects: enable

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 rgba;

layout(location = 0) out vec2 vary_uv;
layout(location = 1) out vec4 vary_rgba;

layout(binding = 0) uniform UniformBufferObject {
  mat4 projection;
} ubo;

void main() {
    vary_uv = uv;
    vary_rgba = rgba;
    gl_Position = ubo.projection * vec4(pos, 0.0, 1.0);
}
