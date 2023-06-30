#version 410 core

#define PI 3.1415926538

#define RADIUS 1.0
#define CIRCLE_POINTS 20

layout(points) in;
layout(triangle_strip, max_vertices = 256) out;

in vec3 position[];

uniform mat4 projection;
uniform mat4 view;

void main() {
    vec3 points[CIRCLE_POINTS + 1];

    for (uint i = 0; i <= CIRCLE_POINTS; i++) {
        float angle = 2 * PI / CIRCLE_POINTS * i;

        points[i] = vec3(RADIUS * cos(angle), 0.0, RADIUS * sin(angle));
    }

    vec4 middle = projection * view * vec4(position[0], 1.0);

    for (uint i = 0; i < points.length(); i++) {
        gl_Position = middle;
        EmitVertex();

        gl_Position = projection * view
            * vec4(points[i] + position[0], 1.0);
        EmitVertex();

        gl_Position = projection * view
            * vec4(points[i + 1] + position[0], 1.0);
        EmitVertex();
    }

    EndPrimitive();
}
