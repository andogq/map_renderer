#version 410 core

layout(location = 0) in vec3 in_position;
layout(location = 1) in float in_width;

out VertexData {
    vec3 position;
    float width;
} out_data;

void main() {
    out_data.position = in_position;
    out_data.width = in_width;
}
