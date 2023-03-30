#version 410 core

layout(location = 0) in uint line_id;
layout(location = 1) in vec3 position;

out VertexData {
    uint line_id;
    vec3 position;
} out_data;

void main() {
    out_data.line_id = line_id;
    out_data.position = position;
}
