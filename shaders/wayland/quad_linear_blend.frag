#version 300 es
precision highp float;

/*
 * quad_linear_blend.frag
 * Honey Wayland Compositor Fragment Shader for Linear Gamma Blending
 *
 * Implements Apple SkyLight / Metal UberCompositeFragment parity in OpenGL ES 3.0 / Vulkan.
 * Blends window surfaces and text alpha masks in linear photometric luminance (gamma = 1.0)
 * before hardware scanout, preventing edge luminance loss on dark backgrounds.
 */

uniform sampler2D u_background;
uniform sampler2D u_source;
uniform float u_opacity;

in vec2 v_texcoord;
out vec4 fragColor;

float srgb_to_linear(float c) {
    return (c <= 0.04045) ? (c / 12.92) : pow((c + 0.055) / 1.055, 2.4);
}

float linear_to_srgb(float lin) {
    return (lin <= 0.0031308) ? (lin * 12.92) : (1.055 * pow(lin, 1.0 / 2.4) - 0.055);
}

void main() {
    vec4 bg = texture(u_background, v_texcoord);
    vec4 src = texture(u_source, v_texcoord);

    float alpha = src.a * u_opacity;
    if (alpha <= 0.0) {
        fragColor = bg;
        return;
    }

    /* Transform background and foreground to linear photometric luminance */
    vec3 bg_lin = vec3(
        srgb_to_linear(bg.r),
        srgb_to_linear(bg.g),
        srgb_to_linear(bg.b)
    );

    vec3 fg_lin = vec3(
        srgb_to_linear(src.r),
        srgb_to_linear(src.g),
        srgb_to_linear(src.b)
    );

    /* Photometric blend in linear energy space (gamma = 1.0) */
    vec3 out_lin = fg_lin * alpha + bg_lin * (1.0 - alpha);

    /* Reconvert to display space (sRGB) */
    fragColor = vec4(
        linear_to_srgb(out_lin.r),
        linear_to_srgb(out_lin.g),
        linear_to_srgb(out_lin.b),
        bg.a + alpha * (1.0 - bg.a)
    );
}
