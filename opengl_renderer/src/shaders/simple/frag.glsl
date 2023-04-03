#version 410 core

in vec3 color;
out vec4 frag_color;

uniform float blue;

void main () {
    frag_color = vec4(color.xy, blue, 1.0);
}
