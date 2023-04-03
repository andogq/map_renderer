use super::OpenGlError;
use glam::Mat4;
use opengl::{Buffer, BufferType, Capability, Context, Location, ShaderType, VertexArrayObject};
use std::{
    cell::RefCell, collections::HashMap, error::Error, fmt::Display, fs, path::Path, rc::Rc,
};

pub trait UniformValue {
    fn set_uniform(&self, gl: &Context, location: Location);
}

impl UniformValue for f32 {
    fn set_uniform(&self, gl: &Context, location: Location) {
        gl.uniform_f32(location, *self)
    }
}

impl UniformValue for Mat4 {
    fn set_uniform(&self, gl: &Context, location: Location) {
        gl.uniform_mat4(location, self)
    }
}

#[derive(Clone, Copy)]
pub enum DrawType {
    Triangles,
    Points,
    Lines,
    LineStrip,
}
impl From<DrawType> for opengl::DrawType {
    fn from(value: DrawType) -> Self {
        match value {
            DrawType::Triangles => opengl::DrawType::Triangles,
            DrawType::Points => opengl::DrawType::Points,
            DrawType::Lines => opengl::DrawType::Lines,
            DrawType::LineStrip => opengl::DrawType::LineStrip,
        }
    }
}

#[derive(Debug)]
pub enum ProgramBuilderError {
    IoError(std::io::Error),
    MissingGl,
    OpenGlError(opengl::OpenGlError),
    CreateProgram(String),
    CreateShader(String),
    ShaderCompile(String),
    CreateVertexBuffer(String),
    CreateVertexArray(String),
    ProgramLink(String),
}
impl Display for ProgramBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem building shader: {:?}", self)
    }
}
impl Error for ProgramBuilderError {}

#[derive(Default)]
pub struct ProgramBuilder {
    gl: Option<Rc<RefCell<Context>>>,
    shaders: Vec<(ShaderType, String)>,
    vertex_format: Vec<VertexFormat>,
    draw_type: Option<DrawType>,
    buffer_texture: bool,
}
impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_gl(mut self, gl: Rc<RefCell<Context>>) -> Self {
        self.gl = Some(gl);
        self
    }

    pub fn with_shader(mut self, shader_type: ShaderType, source: &str) -> Self {
        self.shaders.push((shader_type, source.to_string()));
        self
    }

    pub fn with_format(mut self, format: &[VertexFormat]) -> Self {
        self.vertex_format = format.to_vec();
        self
    }

    pub fn with_draw_type(mut self, draw_type: DrawType) -> Self {
        self.draw_type = Some(draw_type);
        self
    }

    pub fn with_buffer_texture(mut self) -> Self {
        self.buffer_texture = true;
        self
    }

    pub fn build(self) -> Result<Program, ProgramBuilderError> {
        let gl = self.gl.ok_or(ProgramBuilderError::MissingGl)?;

        let (program, vertex_buffer, vertex_array_object, texture_buffer) = {
            let gl = gl.borrow();

            // Create the opengl program
            let program = gl
                .create_program()
                .map_err(ProgramBuilderError::OpenGlError)?;

            // Build the shaders
            let shaders = self
                .shaders
                .into_iter()
                .map(|(shader_type, source)| {
                    gl.create_shader(shader_type)
                        .map_err(ProgramBuilderError::OpenGlError)
                        .and_then(|shader| {
                            // Add source and compile
                            gl.shader_source(shader, &source);
                            gl.compile_shader(shader);

                            // Check for errors
                            if !gl.get_shader_compile_status(shader) {
                                return Err(ProgramBuilderError::ShaderCompile(
                                    gl.get_shader_info_log(shader),
                                ));
                            }

                            // Add shader to program
                            gl.attach_shader(program, shader);

                            Ok(shader)
                        })
                })
                .collect::<Result<Vec<_>, ProgramBuilderError>>()?;

            // Link the program
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(ProgramBuilderError::ProgramLink(
                    gl.get_program_info_log(program),
                ));
            }

            // Clean up shaders
            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            // Create the vertex buffer object and bind it so it can be detected by VAO
            let vertex_buffer = gl
                .create_buffer()
                .map_err(ProgramBuilderError::OpenGlError)?;
            gl.bind_buffer(BufferType::ArrayBuffer, vertex_buffer);

            // Calculate the step size of vertex, and the offsets for each part of it
            let (vertex_step, offsets) = self.vertex_format.iter().fold(
                (0, {
                    let mut v = Vec::with_capacity(self.vertex_format.len());
                    v.push(0);
                    v
                }),
                |(size, mut offsets), format| {
                    offsets.push(offsets.last().cloned().unwrap_or(0) + format.size());
                    (size + format.size(), offsets)
                },
            );

            // Create vertex array object to save state
            let vertex_array_object = gl
                .create_vertex_array()
                .map_err(ProgramBuilderError::OpenGlError)?;
            gl.bind_vertex_array(vertex_array_object);

            // Configure all of the attributes
            for (i, (format, offset)) in self.vertex_format.iter().zip(offsets).enumerate() {
                gl.enable_vertex_attribute_array(i as u32);
                gl.vertex_attribute_pointer_f32(
                    i as u32,
                    format.count,
                    (&format.vertex_type).into(),
                    false,
                    vertex_step,
                    offset,
                );
            }

            let mut buffer_texture = None;
            if self.buffer_texture {
                // Create the vertex buffer
                buffer_texture = Some(gl.create_buffer().unwrap());

                // TODO: Bind the texture buffer
            }

            (program, vertex_buffer, vertex_array_object, buffer_texture)
        };

        // Build the program
        Ok(Program {
            program,
            gl,
            vertex_buffer,
            vertex_array_object,
            vertex_count: None,
            vertex_format: self.vertex_format,
            uniform_locations: HashMap::new(),
            draw_type: self.draw_type.unwrap_or(DrawType::Triangles),
            draw_arrays: None,
            texture_buffer,
        })
    }
}

pub struct DrawArrays {
    first: Vec<u32>,
    count: Vec<u32>,
}
impl DrawArrays {
    pub fn new(first: Vec<u32>, count: Vec<u32>) -> Self {
        Self { first, count }
    }

    pub fn new_continuous(count: Vec<u32>) -> Self {
        // Really yucky
        let mut first = Vec::with_capacity(count.len());
        first.push(0);

        for i in 1..count.len() {
            first.push(first[i - 1] + count[i - 1]);
        }

        Self { first, count }
    }
}

pub trait VertexData {
    fn get_bytes(&self) -> Vec<u8>;
}

impl<V> VertexData for &[V]
where
    V: VertexData,
{
    fn get_bytes(&self) -> Vec<u8> {
        self.iter().flat_map(|v| v.get_bytes()).collect()
    }
}

impl VertexData for Vec<u8> {
    fn get_bytes(&self) -> Vec<u8> {
        self.clone()
    }
}

pub struct Program {
    program: opengl::Program,
    gl: Rc<RefCell<Context>>,
    vertex_buffer: Buffer,
    vertex_array_object: VertexArrayObject,
    vertex_count: Option<u32>,
    vertex_format: Vec<VertexFormat>,
    uniform_locations: HashMap<String, Location>,
    draw_type: DrawType,
    draw_arrays: Option<DrawArrays>,
    texture_buffer: Option<Buffer>,
}

impl Program {
    pub fn builder() -> ProgramBuilder {
        ProgramBuilder::new()
    }

    pub fn from_directory(directory_name: &str) -> Result<ProgramBuilder, ProgramBuilderError> {
        let mut builder = ProgramBuilder::new();

        // Read the directory
        let directory = Path::new("opengl_renderer/src/shaders").join(directory_name);
        for entry in fs::read_dir(directory).map_err(ProgramBuilderError::IoError)? {
            let entry = entry.map_err(ProgramBuilderError::IoError)?;

            // Check if file
            if entry
                .file_type()
                .map_err(ProgramBuilderError::IoError)?
                .is_file()
            {
                // Check if valid file type
                let stripped_name = entry.file_name().to_string_lossy().replace(".glsl", "");

                if let Some(shader_type) = match stripped_name.as_str() {
                    "vert" | "vertex" => Some(ShaderType::Vertex),
                    "geom" | "geometry" => Some(ShaderType::Geometry),
                    "frag" | "fragment" => Some(ShaderType::Fragment),
                    _ => None,
                } {
                    // Read file contents
                    let source =
                        fs::read_to_string(entry.path()).map_err(ProgramBuilderError::IoError)?;

                    builder = builder.with_shader(shader_type, source.as_str());
                }
            }
        }

        Ok(builder)
    }

    pub fn use_program(&self) {
        let gl = self.gl.borrow();
        gl.use_program(self.program);
    }

    pub fn render(&self) {
        self.use_program();

        let gl = self.gl.borrow();
        gl.enable(Capability::ProgramPointSize);
        if let Some(vertex_count) = self.vertex_count {
            // Rebind vertex array
            gl.bind_vertex_array(self.vertex_array_object);

            if let Some(DrawArrays { first, count }) = self.draw_arrays.as_ref() {
                // glow doesn't support glMultiDrawArrays, but *alegedly* this has the same
                // performance impact
                for (&first, &count) in first.iter().zip(count.iter()) {
                    gl.draw_arrays(self.draw_type.into(), first, count);
                }
            } else {
                gl.draw_arrays(self.draw_type.into(), 0, vertex_count);
            }
        }
    }

    pub fn attach_vertices(
        &mut self,
        vertices: impl VertexData,
        draw_arrays: Option<DrawArrays>,
    ) -> Result<(), OpenGlError> {
        self.use_program();

        let gl = self.gl.borrow();

        let vertices = vertices.get_bytes();

        // Bind vertex buffer and add data
        gl.bind_vertex_array(self.vertex_array_object);
        gl.bind_buffer(BufferType::ArrayBuffer, self.vertex_buffer);
        gl.buffer_data_u8_slice(
            BufferType::ArrayBuffer,
            // vertices_u8,
            &vertices,
            opengl::Usage::StaticDraw,
        );

        // TODO: Will change when take other slices
        self.vertex_count = Some(
            (vertices.len() as u32)
                / self
                    .vertex_format
                    .iter()
                    .fold(0, |total_size, format| total_size + format.size()),
        );

        self.draw_arrays = draw_arrays;

        Ok(())
    }

    pub fn set_uniform(
        &mut self,
        name: &str,
        value: &impl UniformValue,
    ) -> Result<(), OpenGlError> {
        self.use_program();

        let gl = self.gl.borrow();

        let location = if let Some(location) = self.uniform_locations.get(name) {
            location
        } else if let Some(location) = gl.get_uniform_location(self.program, name) {
            // Use entry API to get a reference owned by the hashmap, like in the previous branch
            self.uniform_locations
                .entry(name.to_string())
                .or_insert(location)
        } else {
            return Err(OpenGlError::UniformNotFound(name.to_string()));
        };

        value.set_uniform(&gl, *location);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VertexType {
    Float,
    UInt,
}
impl VertexType {
    pub fn size(&self) -> u32 {
        match self {
            VertexType::Float => 4,
            VertexType::UInt => 4,
        }
    }
}
impl From<&VertexType> for opengl::DataType {
    fn from(vertex_type: &VertexType) -> Self {
        match vertex_type {
            VertexType::Float => opengl::DataType::Float,
            VertexType::UInt => opengl::DataType::UnsignedInt,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VertexFormat {
    count: u32,
    vertex_type: VertexType,
}
impl VertexFormat {
    pub fn new(count: u32, vertex_type: VertexType) -> Self {
        VertexFormat { count, vertex_type }
    }
    pub fn size(&self) -> u32 {
        self.count * self.vertex_type.size()
    }
}
