#version 450
#extension GL_ARB_separate_shader_objects: enable

layout(location = 0) in vec2 vary_uv;
layout(location = 0) out vec4 frag_color;

void main() {
    frag_color = vec4(0.0, vary_uv.x, vary_uv.y, 1.0);
}
