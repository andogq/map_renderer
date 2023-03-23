#version 410 core

layout(lines) in;

in VertexData {
    vec3 position;
    float width;
    vec3 color;
} in_data[];

layout(triangle_strip, max_vertices = 4) out;
// layout(points, max_vertices = 6) out;

out vec3 color;

uniform mat4 projection;
uniform mat4 view;

void main() {
    gl_PointSize = 10.0;

    vec3 l = in_data[1].position - in_data[0].position;
    vec3 l_perp = normalize(vec3(l.z, l.y, -l.x));
    vec3 normalized_l = normalize(l);

    mat4 pv = projection * view;

    // For start and end of line
    for (int i = 0; i < 2; i++) {
        // gl_Position = pv * vec4(in_data[i].position, 1.0);
        // color = vec3(0.0, 0.0, 0.0);
        // EmitVertex();

        // For 'up' and 'down' direction
        for (int dir = -1; dir <= 1; dir += 2) {
            float w = in_data[i].width / 2;

            vec3 p = in_data[i].position
                // Move perpendicular to line
                + (l_perp * w * dir)
                // Move parallel to line
                + (normalized_l * w * (1 - (2 * (1 - i))));

            gl_Position = pv * vec4(p, 1.0);
            color = in_data[i].color;
            EmitVertex();
        }
    }

    EndPrimitive();
}
