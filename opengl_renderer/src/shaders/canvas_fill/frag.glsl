#version 410 core

in VertexData {
    vec3 color;
} in_data;

out vec4 frag_color;

void main () {
    frag_color = vec4(in_data.color, 1.0);
}
