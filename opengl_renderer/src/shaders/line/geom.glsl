#version 410 core

layout(lines) in;
in vec3 position[2];

layout(triangle_strip, max_vertices = 6) out;
out vec3 colors;

uniform mat4 projection;
uniform mat4 view;

void main() {
    float width = 1.0;

    vec3 l = position[1] - position[0];
    vec3 l_perp = normalize(vec3(l.z, l.y, -l.x));

    vec3 p1 = position[0] + (l_perp * width / 2);
    vec3 p2 = position[0] - (l_perp * width / 2);
    vec3 p3 = position[1] + (l_perp * width / 2);
    vec3 p4 = position[1] - (l_perp * width / 2);

    gl_Position = projection * view * vec4(p1, 1.0);
    colors = vec3(1.0, 0.0, 0.0);
    EmitVertex();

    gl_Position = projection * view * vec4(p2, 1.0);
    colors = vec3(1.0, 0.0, 0.0);
    EmitVertex();

    gl_Position = projection * view * vec4(p3, 1.0);
    colors = vec3(1.0, 0.0, 0.0);
    EmitVertex();

    gl_Position = projection * view * vec4(p4, 1.0);
    colors = vec3(1.0, 0.0, 0.0);
    EmitVertex();

    EndPrimitive();
}
