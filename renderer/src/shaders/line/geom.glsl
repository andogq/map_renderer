#version 410 core

layout(lines) in;

in VertexData {
    uint line_id;
    vec3 position;
    float width;
    vec3 color;
    float stroke_length;
} in_data[];

layout(triangle_strip, max_vertices = 100) out;
// layout(points, max_vertices = 100) out;

out vec3 color;

uniform mat4 projection;
uniform mat4 view;

void main() {
    gl_PointSize = 5.0;

    vec3 l = in_data[1].position - in_data[0].position;
    vec3 l_perp = normalize(vec3(l.z, l.y, -l.x));
    vec3 normalized_l = normalize(l);

    mat4 pv = projection * view;

    float line_length = length(l);

    float stroke_length = in_data[0].stroke_length;
    if (stroke_length == 0) stroke_length = line_length;

    int stroke_count = int(ceil(line_length / stroke_length));

    if (in_data[0].line_id == in_data[1].line_id) {
        // Generate segments
        for (int stroke_i = 0; stroke_i <= stroke_count; stroke_i += 1) {
            vec3 position = in_data[0].position + (stroke_i * stroke_length * normalized_l);

            if (distance(position, in_data[0].position) > length(in_data[1].position)) {
                position = in_data[1].position;
            }

            for (int dir = -1; dir <= 1; dir += 2) {
                float w = in_data[0].width / 2;

                vec3 p = position
                    // Move perpendicular to line
                    + (l_perp * w * dir);

                // TODO: Give offcut to next connected line

                bool first = stroke_i == 0;
                bool last = stroke_i == stroke_count;
                if (first || last) {
                    // Move parallel to line
                    p += (normalized_l * w * (first ? -1 : 1));
                }

                gl_Position = pv * vec4(p, 1.0);

                color = in_data[0].color;

                EmitVertex();
            }

            if (stroke_i % 2 == 1) EndPrimitive();
        }
    }
}
