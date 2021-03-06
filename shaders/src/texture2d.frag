#version 450
#extension GL_ARB_separate_shader_objects: enable

layout(constant_id = 0) const uint MAX_TEXTURES = 1;
layout(binding = 0) uniform sampler2D textures[MAX_TEXTURES];

layout(location = 0) in vec2 vary_uv;
layout(location = 1) in vec4 vary_rgba;

layout(location = 0) out vec4 frag_color;

layout(push_constant) uniform PushConsts {
    mat4 projection;
    uint texture_index;
} pushConsts;

void main() {
    vec4 sampled_value = texture(textures[pushConsts.texture_index], vary_uv);
    frag_color = vary_rgba * sampled_value;
}
