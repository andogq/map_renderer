#version 410 core

layout(location = 0) in vec3 position;
layout(location = 1) in float width;
layout(location = 2) in vec3 color;
layout(location = 3) in uint stroke_length;

out VertexData {
    vec3 position;
    float width;
    vec3 color;
    float stroke_length;
} out_data;

void main() {
    out_data.position = position;
    out_data.width = width;
    out_data.color = color;
    out_data.stroke_length = stroke_length;
}
