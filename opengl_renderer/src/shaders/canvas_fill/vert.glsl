#version 410 core

layout(location = 0) in uint line_id;
layout(location = 1) in vec3 position;

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
    gl_Position = projection * view * vec4(position, 1.0);
    gl_PointSize = line_id;

    PathData path = get_path(line_id);

    if (path.has_fill) {
        out_data.color = path.stroke_color;
    } else {
        out_data.color = vec3(1.0, 0.0, 0.0);
    }
}
