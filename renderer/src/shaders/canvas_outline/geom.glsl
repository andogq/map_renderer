#version 410 core

#define PI 3.1415926538
#define ARC_RESOLUTION 5

layout(lines) in;

in VertexData {
    uint line_id;
    vec3 position;
} in_data[];

layout(triangle_strip, max_vertices = 256) out;
// layout(points, max_vertices = 256) out;

out vec3 color;

uniform mat4 projection;
uniform mat4 view;

uniform samplerBuffer path_data;

// Number of 32 bit floats per each path data struct
const uint PATH_DATA_SIZE = 9;

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

    gl_PointSize = 10.0;

    vec3 l = end - start;
    vec3 l_perp = normalize(vec3(l.z, l.y, -l.x));
    vec3 normalized_l = normalize(l);

    mat4 pv = projection * view;

    float line_length = length(l);

    // Default stroke dash if none is provided
    if (path.stroke_dash == 0.0) path.stroke_dash = line_length;

    int stroke_count = int(ceil(line_length / path.stroke_dash));

    // Use same color for line
    color = path.stroke_color;

    // Generate segments
    for (float stroke_i = 0.5; stroke_i < float(stroke_count); stroke_i += 2.0) {
        vec3 line_middle = start + (float(stroke_i) * path.stroke_dash * normalized_l);

        // Generate corners
        vec3 corners[4];
        uint i = 0;
        for (int dx = -1; dx <= 1; dx += 2) {
            for (int dy = -1; dy <= 1; dy += 2) {
                vec3 p = line_middle
                    + (l_perp * path.stroke_width * 0.5 * dx) // Add width
                    + (normalized_l * dy * path.stroke_dash * 0.5); // Add length

                corners[i] = p;
                i++;
            }
        }

        // Emit corners
        int indices[] = int[]( 0, 1, 2, 1, 2, 3 );
        for (uint i = 0; i < indices.length(); i++) {
            gl_Position = pv * vec4(corners[i], 1.0);
            EmitVertex();
        }

        EndPrimitive();

        // Generate rounded caps
        for (int dir = -1; dir <= 1; dir += 2) {
            vec3 middle = line_middle + (normalized_l * path.stroke_dash * 0.5 * dir);

            color = vec3(1.0, 0.0, 0.0);
            gl_Position = pv * vec4(middle, 1.0);

            vec3 arc_points[ARC_RESOLUTION + 1];

            for (uint i = 0; i <= ARC_RESOLUTION + 1; i++) {
                float angle = PI / ARC_RESOLUTION * i;

                vec3 p = middle
                    + (l_perp * path.stroke_width * 0.5 * cos(angle))
                    + (normalized_l * path.stroke_width * 0.5 * dir * sin(angle));

                arc_points[i] = p;
            }

            color = path.stroke_color;

            // Emit vertices for caps
            for (uint i = 0; i < arc_points.length() - 1; i++) {
                vec3 points[] = vec3[]( middle, arc_points[i], arc_points[i + 1] );

                for (int point_i = 0; point_i < points.length(); point_i++) {
                    gl_Position = pv * vec4(points[point_i], 1.0);
                    EmitVertex();
                }
            }

            EndPrimitive();
        }
    }
}
