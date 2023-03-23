use glam::Mat4;
use glow::{
    Context, HasContext, NativeBuffer, NativeProgram, NativeShader, NativeVertexArray,
    UniformLocation, ARRAY_BUFFER,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fmt::Display,
    fs::{self, FileType},
    path::{Path, PathBuf},
    rc::Rc,
};

use super::OpenGlError;

pub enum ShaderType {
    Vertex,
    Geometry,
    Fragment,
}
impl From<ShaderType> for u32 {
    fn from(ty: ShaderType) -> Self {
        match ty {
            ShaderType::Vertex => glow::VERTEX_SHADER,
            ShaderType::Geometry => glow::GEOMETRY_SHADER,
            ShaderType::Fragment => glow::FRAGMENT_SHADER,
        }
    }
}
impl TryFrom<&str> for ShaderType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "vert" | "vertex" => Ok(Self::Vertex),
            "geom" | "geometry" => Ok(Self::Geometry),
            "frag" | "fragment" => Ok(Self::Fragment),
            _ => Err(()),
        }
    }
}

pub trait UniformValue {
    unsafe fn set_uniform(&self, gl: &Context, location: &UniformLocation);
}

impl UniformValue for f32 {
    unsafe fn set_uniform(&self, gl: &Context, location: &UniformLocation) {
        gl.uniform_1_f32(Some(location), *self)
    }
}

impl UniformValue for Mat4 {
    unsafe fn set_uniform(&self, gl: &Context, location: &UniformLocation) {
        gl.uniform_matrix_4_f32_slice(Some(location), false, &self.to_cols_array())
    }
}

#[derive(Clone, Copy)]
pub enum DrawType {
    Triangles,
    Points,
    Lines,
}
impl From<DrawType> for u32 {
    fn from(value: DrawType) -> Self {
        match value {
            DrawType::Triangles => glow::TRIANGLES,
            DrawType::Points => glow::POINTS,
            DrawType::Lines => glow::LINE_STRIP,
        }
    }
}

#[derive(Debug)]
pub enum ProgramBuilderError {
    IoError(std::io::Error),
    MissingGl,
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

    pub fn build(self) -> Result<Program, ProgramBuilderError> {
        let gl = self.gl.ok_or(ProgramBuilderError::MissingGl)?;

        let (program, vertex_buffer, vertex_array_object) = {
            let gl = gl.borrow();

            // Create the opengl program
            let program =
                unsafe { gl.create_program() }.map_err(ProgramBuilderError::CreateProgram)?;

            // Build the shaders
            let shaders = self
                .shaders
                .into_iter()
                .map(|(shader_type, source)| {
                    unsafe { gl.create_shader(shader_type.into()) }
                        .map_err(ProgramBuilderError::CreateShader)
                        .and_then(|shader| {
                            unsafe {
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
                            }

                            Ok(shader)
                        })
                })
                .collect::<Result<Vec<_>, ProgramBuilderError>>()?;

            // Link the program
            unsafe { gl.link_program(program) };
            if unsafe { !gl.get_program_link_status(program) } {
                return Err(ProgramBuilderError::ProgramLink(unsafe {
                    gl.get_program_info_log(program)
                }));
            }

            // Clean up shaders
            for shader in shaders {
                unsafe {
                    gl.detach_shader(program, shader);
                    gl.delete_shader(shader);
                }
            }

            // Create the vertex buffer object and bind it so it can be detected by VAO
            let vertex_buffer =
                unsafe { gl.create_buffer() }.map_err(ProgramBuilderError::CreateVertexBuffer)?;
            unsafe { gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer)) };

            // Create vertex array object
            let vertex_array_object = unsafe { gl.create_vertex_array() }
                .map_err(ProgramBuilderError::CreateVertexArray)?;
            unsafe { gl.bind_vertex_array(Some(vertex_array_object)) };

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

            // Configure all of the attributes
            for (i, (format, offset)) in self.vertex_format.iter().zip(offsets).enumerate() {
                unsafe {
                    gl.enable_vertex_attrib_array(i as u32);
                    gl.vertex_attrib_pointer_f32(
                        i as u32,
                        format.count as i32,
                        (&format.vertex_type).into(),
                        false,
                        vertex_step as i32,
                        offset as i32,
                    );
                }
            }

            (program, vertex_buffer, vertex_array_object)
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
        let first = {
            let mut v = count.clone();
            v.insert(0, 0); // First vertex occurs at index 0
            v.pop(); // Don't need the last element
            v
        };

        Self { first, count }
    }
}

pub struct Program {
    program: NativeProgram,
    gl: Rc<RefCell<Context>>,
    vertex_buffer: NativeBuffer,
    vertex_array_object: NativeVertexArray,
    vertex_count: Option<u32>,
    vertex_format: Vec<VertexFormat>,
    uniform_locations: HashMap<String, UniformLocation>,
    draw_type: DrawType,
    draw_arrays: Option<DrawArrays>,
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
                if let Ok(shader_type) = ShaderType::try_from(stripped_name.as_str()) {
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
        unsafe { gl.use_program(Some(self.program)) };
    }

    pub fn render(&self) {
        self.use_program();

        let gl = self.gl.borrow();
        unsafe { gl.enable(glow::PROGRAM_POINT_SIZE) };
        if let Some(vertex_count) = self.vertex_count {
            // Rebind vertex array
            unsafe { gl.bind_vertex_array(Some(self.vertex_array_object)) };

            if let Some(DrawArrays { first, count }) = self.draw_arrays.as_ref() {
                // glow doesn't support glMultiDrawArrays, but *alegedly* this has the same
                // performance impact
                for (&first, &count) in first.iter().zip(count.iter()) {
                    unsafe { gl.draw_arrays(self.draw_type.into(), first as i32, count as i32) };
                }
            } else {
                unsafe { gl.draw_arrays(self.draw_type.into(), 0, vertex_count as i32) };
            }
        }
    }

    pub fn attach_vertices(
        &mut self,
        vertices: &[f32],
        draw_arrays: Option<DrawArrays>,
    ) -> Result<(), OpenGlError> {
        self.use_program();

        let gl = self.gl.borrow();

        // Construct the raw pointer
        let vertices_u8 = unsafe {
            core::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * core::mem::size_of::<f32>(),
            )
        };

        // Bind vertex buffer and add data
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STATIC_DRAW);
        };

        // TODO: Will change when take other slices
        self.vertex_count = Some(
            (vertices.len() as u32)
                / self
                    .vertex_format
                    .iter()
                    .fold(0, |total_size, format| total_size + format.count),
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
        } else if let Some(location) = unsafe { gl.get_uniform_location(self.program, name) } {
            // Use entry API to get a reference owned by the hashmap, like in the previous branch
            self.uniform_locations
                .entry(name.to_string())
                .or_insert(location)
        } else {
            return Err(OpenGlError::UniformNotFound(name.to_string()));
        };

        unsafe { value.set_uniform(&gl, location) };

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum VertexType {
    Float,
}
impl VertexType {
    pub fn size(&self) -> u32 {
        match self {
            VertexType::Float => 4,
        }
    }
}
impl From<&VertexType> for u32 {
    fn from(vertex_type: &VertexType) -> Self {
        match vertex_type {
            VertexType::Float => glow::FLOAT,
        }
    }
}

#[derive(Clone, Copy)]
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
