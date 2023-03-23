#version 410 core

in vec3 colors;
out vec4 color;

void main() {
    color = vec4(colors, 1.0);
}
