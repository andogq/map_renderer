macro_rules! gl_enum {
    ($enum_name:ident { $($variation:ident : $gl_mapping:path),* }) => {
        #[derive(Clone, Copy, Debug)]
        pub enum $enum_name {
            $($variation),*
        }

        impl From<$enum_name> for gl::types::GLenum {
            fn from(value: $enum_name) -> Self {
                match value {
                    $($enum_name::$variation => $gl_mapping),*
                }
            }
        }
    };
}

gl_enum!(StringName {
    Vendor: gl::VENDOR,
    Renderer: gl::RENDERER,
    Version: gl::VERSION,
    ShaderLanguageVersion: gl::SHADING_LANGUAGE_VERSION
});

gl_enum!(BufferMask {
    Color: gl::COLOR_BUFFER_BIT,
    Depth: gl::DEPTH_BUFFER_BIT,
    Stencil: gl::STENCIL_BUFFER_BIT
});

gl_enum!(ShaderType {
    Compute: gl::COMPUTE_SHADER,
    Vertex: gl::VERTEX_SHADER,
    TesellationControl: gl::TESS_CONTROL_SHADER,
    TessellationEvaluation: gl::TESS_EVALUATION_SHADER,
    Geometry: gl::GEOMETRY_SHADER,
    Fragment: gl::FRAGMENT_SHADER
});

gl_enum!(ShaderParameter {
    ShaderType: gl::SHADER_TYPE,
    DeleteStatus: gl::DELETE_STATUS,
    CompileStatus: gl::COMPILE_STATUS,
    InfoLogLength: gl::INFO_LOG_LENGTH,
    ShaderSourceLength: gl::SHADER_SOURCE_LENGTH
});

gl_enum!(ProgramParameter {
    DeleteStatus: gl::DELETE_STATUS,
    LinkStatus: gl::LINK_STATUS,
    ValidateStatus: gl::VALIDATE_STATUS,
    InfoLogLength: gl::INFO_LOG_LENGTH,
    AttachedShaders: gl::ATTACHED_SHADERS,
    ActiveAtomicCounterBuffers: gl::ACTIVE_ATOMIC_COUNTER_BUFFERS,
    ActiveAttributes: gl::ACTIVE_ATTRIBUTES,
    ActiveAttributeMaxLength: gl::ACTIVE_ATTRIBUTE_MAX_LENGTH,
    ActiveUniforms: gl::ACTIVE_UNIFORMS,
    ActiveUniformBlocks: gl::ACTIVE_UNIFORM_BLOCKS,
    ActiveUniformBlockMaxNameLength: gl::ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH,
    ActiveUniformMaxLength: gl::ACTIVE_UNIFORM_MAX_LENGTH,
    ComputeWorkGroupSize: gl::COMPUTE_WORK_GROUP_SIZE,
    ProgramBinaryLength: gl::PROGRAM_BINARY_LENGTH,
    TransformFeedbackBufferMode: gl::TRANSFORM_FEEDBACK_BUFFER_MODE,
    TransformFeedbackVaryings: gl::TRANSFORM_FEEDBACK_VARYINGS,
    TransformFeedbackVaryingMaxLength: gl::TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH,
    GeometryVerticesOut: gl::GEOMETRY_VERTICES_OUT,
    GeometryInputType: gl::GEOMETRY_INPUT_TYPE,
    GlGeometryOutputType: gl::GEOMETRY_OUTPUT_TYPE
});

gl_enum!(BufferType {
    ArrayBuffer: gl::ARRAY_BUFFER,
    AtomicCounterBuffer: gl::ATOMIC_COUNTER_BUFFER,
    CopyReadBuffer: gl::COPY_READ_BUFFER,
    CopyWriteBuffer: gl::COPY_WRITE_BUFFER,
    DispatchIndirectBuffer: gl::DISPATCH_INDIRECT_BUFFER,
    DrawIndirectBuffer: gl::DRAW_INDIRECT_BUFFER,
    ElementArrayBuffer: gl::ELEMENT_ARRAY_BUFFER,
    PixelPackBuffer: gl::PIXEL_PACK_BUFFER,
    PixelUnpackBuffer: gl::PIXEL_UNPACK_BUFFER,
    QueryBuffer: gl::QUERY_BUFFER,
    ShaderStorageBuffer: gl::SHADER_STORAGE_BUFFER,
    TextureBuffer: gl::TEXTURE_BUFFER,
    TransformFeedbackBuffer: gl::TRANSFORM_FEEDBACK_BUFFER,
    UniformBuffer: gl::UNIFORM_BUFFER
});

gl_enum!(DataType {
    Byte: gl::BYTE,
    UnsignedByte: gl::UNSIGNED_BYTE,
    Short: gl::SHORT,
    UnsignedShort: gl::UNSIGNED_SHORT,
    Int: gl::INT,
    UnsignedInt: gl::UNSIGNED_INT,
    HalfFloat: gl::HALF_FLOAT,
    Float: gl::FLOAT,
    Double: gl::DOUBLE,
    Fixed: gl::FIXED,
    Int2_10_10_10Rev: gl::INT_2_10_10_10_REV,
    UnsignedInt2_10_10_10Rev: gl::UNSIGNED_INT_2_10_10_10_REV,
    UnsignedInt10f11f11fRev: gl::UNSIGNED_INT_10F_11F_11F_REV
});

gl_enum!(Capability {
    Blend: gl::BLEND,
    ClipDistance: gl::CLIP_DISTANCE0, // WARN: Unsure about value for this
    ColorLogicOp: gl::COLOR_LOGIC_OP,
    CullFace: gl::CULL_FACE,
    DebugOutput: gl::DEBUG_OUTPUT,
    DebugOutputSynchronous: gl::DEBUG_OUTPUT_SYNCHRONOUS,
    DepthClamp: gl::DEPTH_CLAMP,
    DepthTest: gl::DEPTH_TEST,
    Dither: gl::DITHER,
    FramebufferSrgb: gl::FRAMEBUFFER_SRGB,
    LineSmooth: gl::LINE_SMOOTH,
    Multisample: gl::MULTISAMPLE,
    PolygonOffsetFill: gl::POLYGON_OFFSET_FILL,
    PolygonOffsetLine: gl::POLYGON_OFFSET_LINE,
    PolygonOffsetPoint: gl::POLYGON_OFFSET_POINT,
    PolygonSmooth: gl::POLYGON_SMOOTH,
    PrimitiveRestart: gl::PRIMITIVE_RESTART,
    PrimitiveRestartFixedIndex: gl::PRIMITIVE_RESTART_FIXED_INDEX,
    RasterizerDiscard: gl::RASTERIZER_DISCARD,
    SampleAlphaToCoverage: gl::SAMPLE_ALPHA_TO_COVERAGE,
    SampleAlphaToOne: gl::SAMPLE_ALPHA_TO_ONE,
    SampleCoverage: gl::SAMPLE_COVERAGE,
    SampleShading: gl::SAMPLE_SHADING,
    SampleMask: gl::SAMPLE_MASK,
    ScissorTest: gl::SCISSOR_TEST,
    StencilTest: gl::STENCIL_TEST,
    TextureCubeMapSeamless: gl::TEXTURE_CUBE_MAP_SEAMLESS,
    ProgramPointSize: gl::PROGRAM_POINT_SIZE
});

gl_enum!(DrawType {
    Points: gl::POINTS,
    LineStrip: gl::LINE_STRIP,
    LineLoop: gl::LINE_LOOP,
    Lines: gl::LINES,
    LineStripAdjacency: gl::LINE_STRIP_ADJACENCY,
    LinesAdjacency: gl::LINES_ADJACENCY,
    TriangleStrip: gl::TRIANGLE_STRIP,
    TriangleFan: gl::TRIANGLE_FAN,
    Triangles: gl::TRIANGLES,
    TriangleStripAdjacency: gl::TRIANGLE_STRIP_ADJACENCY,
    TrianglesAdjacency: gl::TRIANGLES_ADJACENCY,
    Patches: gl::PATCHES
});

gl_enum!(Usage {
    StreamDraw: gl::STREAM_DRAW,
    StreamRead: gl::STREAM_READ,
    StreamCopy: gl::STREAM_COPY,
    StaticDraw: gl::STATIC_DRAW,
    StaticRead: gl::STATIC_READ,
    StaticCopy: gl::STATIC_COPY,
    DynamicDraw: gl::DYNAMIC_DRAW,
    DynamicRead: gl::DYNAMIC_READ,
    DynamicCopy: gl::DYNAMIC_COPY
});

gl_enum!(ImageFormat {
    R8: gl::R8,
    R16: gl::R16,
    R16F: gl::R16F,
    R32F: gl::R32F,
    R8I: gl::R8I,
    R16I: gl::R16I,
    R32I: gl::R32I,
    R8UI: gl::R8UI,
    R16UI: gl::R16UI,
    R32UI: gl::R32UI,
    RG8: gl::RG8,
    RG16: gl::RG16,
    RG16F: gl::RG16F,
    RG32F: gl::RG32F,
    RG8I: gl::RG8I,
    RG16I: gl::RG16I,
    RG32I: gl::RG32I,
    RG8UI: gl::RG8UI,
    RG16UI: gl::RG16UI,
    RG32UI: gl::RG32UI,
    RGB32F: gl::RGB32F,
    RGB32I: gl::RGB32I,
    RGB32UI: gl::RGB32UI,
    RGBA8: gl::RGBA8,
    RGBA16: gl::RGBA16,
    RGBA16F: gl::RGBA16F,
    RGBA32F: gl::RGBA32F,
    RGBA8I: gl::RGBA8I,
    RGBA16I: gl::RGBA16I,
    RGBA32I: gl::RGBA32I,
    RGBA8UI: gl::RGBA8UI,
    RGBA16UI: gl::RGBA16UI,
    RGBA32UI: gl::RGBA32UI
});

gl_enum!(TextureTarget {
    _1d: gl::TEXTURE_1D,
    _2d: gl::TEXTURE_2D,
    _3d: gl::TEXTURE_3D,
    _1dArray: gl::TEXTURE_1D_ARRAY,
    _2dArray: gl::TEXTURE_2D_ARRAY,
    Rectangle: gl::TEXTURE_RECTANGLE,
    CubeMap: gl::TEXTURE_CUBE_MAP,
    CubeMapArray: gl::TEXTURE_CUBE_MAP_ARRAY,
    Buffer: gl::TEXTURE_BUFFER,
    _2dMultisample: gl::TEXTURE_2D_MULTISAMPLE,
    _2dMultisampleArray: gl::TEXTURE_2D_MULTISAMPLE_ARRAY
});
