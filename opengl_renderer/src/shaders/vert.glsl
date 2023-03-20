#version 410 core

layout(location = 0) in vec3 position;

out vec3 out_position;

uniform mat4 projection;
uniform mat4 view;

void main() {
    out_position = position;
    gl_Position = projection * view * vec4(position, 1.0);
    gl_PointSize = 5.0;
}
