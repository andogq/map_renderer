#version 410 core

layout(location = 0) in vec3 position;
layout(location = 1) in float width;
layout(location = 2) in vec3 color;

out VertexData {
    vec3 position;
    float width;
    vec3 color;
} out_data;

void main() {
    out_data.position = position;
    out_data.width = width;
    out_data.color = color;
}
