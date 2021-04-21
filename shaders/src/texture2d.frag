#version 450
#extension GL_ARB_separate_shader_objects: enable

layout(location = 0) in vec2 vary_uv;
layout(location = 1) in vec4 vary_rgba;

layout(location = 0) out vec4 frag_color;

layout(push_constant) uniform PushConsts {
    uint texture_index;
} pushConsts;

layout(binding = 1) uniform sampler2D textures[80];

void main() {
    vec4 sampled_value = texture(textures[pushConsts.texture_index], vary_uv);
    frag_color = vary_rgba * sampled_value;
}
