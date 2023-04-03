#version 410 core

layout(location = 0) in uint line_id;
layout(location = 1) in vec3 position;

uniform mat4 projection;
uniform mat4 view;

out VertexData {
    vec3 color;
} out_data;

void main() {
    gl_Position = projection * view * vec4(position, 1.0);
    gl_PointSize = line_id;

    out_data.color = vec3(0.2, 0.5, 0.9);
}
