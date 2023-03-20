#version 410 core

in vec3 out_position;
out vec4 color;

void main() {
    if (out_position.x == 0) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else if (out_position.z == 0) {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    } else {
        color = vec4(0.6, 0.6, 0.6, 1.0);
    }
}
