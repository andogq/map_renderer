#version 410 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

uniform mat4 view;
uniform mat4 projection;

out VertexData {
    vec3 color;
} out_data;

void main() {
    gl_Position = projection * view * vec4(position, 1.0);
    gl_PointSize = 10;
    out_data.color = color;
}
