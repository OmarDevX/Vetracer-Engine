#version 460 core
layout (location = 0) in vec2 in_pos;
out vec2 tex_coords;
void main() {
    tex_coords = in_pos * 0.5 + 0.5;
    gl_Position = vec4(in_pos, 0.0, 1.0);
}