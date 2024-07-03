#version 460 core
in vec2 tex_coords;
out vec4 frag_color;
uniform sampler2D screen_texture;
void main() {
    frag_color = texture(screen_texture, tex_coords);
}