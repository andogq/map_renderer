#version 410 core

layout(location = 0) in vec3 in_position;

out vec3 position;

void main() {
    position = in_position;
}
