#version 450
#extension GL_ARB_separate_shader_objects: enable

// Default to 80, this gets overridden to a bigger or smaller value when the
// shader is actually built into a graphics pipeline
layout(constant_id = 0) const uint MAX_TEXTURES = 2;

layout(location = 0) in vec2 vary_uv;
layout(location = 1) in vec4 vary_rgba;

layout(location = 0) out vec4 frag_color;

layout(push_constant) uniform PushConsts {
    uint texture_index;
} pushConsts;

layout(binding = 1) uniform sampler2D textures[MAX_TEXTURES];

void main() {
    vec4 sampled_value = texture(textures[pushConsts.texture_index], vary_uv);
    frag_color = vary_rgba * sampled_value;
}
