#version 410 core

layout(location = 0) in vec3 position;

out vec3 color;

void main() {
    gl_Position = vec4(position, 1.0);
    gl_PointSize = 10;

    color = position;
}
