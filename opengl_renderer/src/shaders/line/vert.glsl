#version 410 core

layout(location = 0) in uint line_id;
layout(location = 1) in vec3 position;
layout(location = 2) in float width;
layout(location = 3) in vec3 color;
layout(location = 4) in uint stroke_length;

out VertexData {
    uint line_id;
    vec3 position;
    float width;
    vec3 color;
    float stroke_length;
} out_data;

void main() {
    out_data.line_id = line_id;
    out_data.position = position;
    out_data.width = width;
    out_data.color = color;
    out_data.stroke_length = stroke_length;
}
