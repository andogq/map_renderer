#version 410 core

layout(location = 0) in vec3 in_position;

out vec3 position;

uniform mat4 projection;
uniform mat4 view;

void main() {
    position = in_position;

    gl_Position = projection * view * vec4(in_position, 1.0);
    gl_PointSize = 5.0;
}
