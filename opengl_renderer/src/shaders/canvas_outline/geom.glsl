#version 410 core

layout(lines) in;

in VertexData {
    uint line_id;
    vec3 position;
} in_data[];

layout(triangle_strip, max_vertices = 100) out;
// layout(points, max_vertices = 100) out;

out vec3 color;

uniform mat4 projection;
uniform mat4 view;

uniform samplerBuffer path_data;

// Number of 32 bit floats per each path data struct
const uint PATH_DATA_SIZE = 7;

out VertexData {
    vec3 color;
} out_data;

float get(uint i) {
    return texelFetch(path_data, int(i)).r;
}

vec3 get_vec3(uint start) {
    return vec3(
        get(start),
        get(start + 1),
        get(start + 2)
    );
}

bool get_bit(uint n, uint bit_number) {
    uint one = 1;
    return ((n >> bit_number) & one) == one;
}

struct PathData {
    // Metadata
    bool has_stroke;
    bool has_fill;

    // Stroke
    vec3 stroke_color;
    float stroke_width;
    float stroke_dash;

    // Fill
    vec3 fill_color;
};

PathData get_path(uint path) {
    uint base = path * PATH_DATA_SIZE;

    uint metadata = floatBitsToUint(get(base));

    bool has_stroke = get_bit(metadata, 1);
    bool has_fill = get_bit(metadata, 0);

    vec3 stroke_color = get_vec3(base + 1);
    float stroke_width = get(base + 4);
    float stroke_dash = get(base + 5);

    vec3 fill_color = get_vec3(base + 6);

    return PathData (
        has_stroke,
        has_fill,
        stroke_color,
        stroke_width,
        stroke_dash,
        fill_color
    );
}

void main() {
    // Don't generate geometry between non-connected lines
    if (in_data[0].line_id != in_data[1].line_id) return;

    // Load path data
    PathData path = get_path(in_data[0].line_id);
    vec3 start = in_data[0].position;
    vec3 end = in_data[1].position;

    gl_PointSize = 5.0;

    vec3 l = end - start;
    vec3 l_perp = normalize(vec3(l.z, l.y, -l.x));
    vec3 normalized_l = normalize(l);

    mat4 pv = projection * view;

    float line_length = length(l);

    // Default stroke dash if none is provided
    if (path.stroke_dash == 0) path.stroke_dash = line_length;

    int stroke_count = int(ceil(line_length / path.stroke_dash));

    // Generate segments
    for (int stroke_i = 0; stroke_i <= stroke_count; stroke_i += 1) {
        vec3 position = start + (stroke_i * path.stroke_dash * normalized_l);

        if (distance(position, start) > distance(start, end)) {
            position = end;
        }

        for (int dir = -1; dir <= 1; dir += 2) {
            float w = path.stroke_width / 2.0;

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
            // color = path_data.stroke_color;
            color = vec3(0.3, 0.5, 0.2);

            EmitVertex();
        }

        if (stroke_i % 2 == 1) EndPrimitive();
    }
}
