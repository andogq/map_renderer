use glow::{
    Context, HasContext, NativeBuffer, NativeProgram, NativeShader, NativeVertexArray,
    UniformLocation,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    rc::Rc,
};

#[derive(Debug)]
pub enum OpenGlError {
    ShaderCreate,
    ShaderCompile(String),
    LinkError,
    BufferCreate(String),
    VertexArrayCreate(String),
    UniformNotFound(String),
}
impl Display for OpenGlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "OpenGl error: {} error",
            match self {
                Self::ShaderCreate => "shader create",
                Self::ShaderCompile(_) => "shader compile",
                Self::LinkError => "link error",
                Self::BufferCreate(_) => "buffer create",
                Self::VertexArrayCreate(_) => "buffer create",
                Self::UniformNotFound(_) => "uniform not found",
            }
        )
    }
}
impl Error for OpenGlError {}

pub struct OpenGlInfo {
    renderer: String,
    version: String,
    shading_language_version: String,
}
impl Debug for OpenGlInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Renderer: {}", self.renderer)?;
        writeln!(f, "Version: {}", self.version)?;
        writeln!(
            f,
            "Shading Language Version: {}",
            self.shading_language_version
        )?;

        Ok(())
    }
}

pub struct OpenGl {
    gl: Rc<RefCell<Context>>,
    programs: Vec<Rc<RefCell<Program>>>,
}

impl OpenGl {
    pub fn new(gl: Context) -> Self {
        Self {
            gl: Rc::new(RefCell::new(gl)),
            programs: Vec::new(),
        }
    }

    pub fn get_info(&self) -> OpenGlInfo {
        let (renderer, version, shading_language_version) = {
            let mut iter = [
                glow::RENDERER,
                glow::VERSION,
                glow::SHADING_LANGUAGE_VERSION,
            ]
            .into_iter()
            .map(|addr| unsafe { self.gl.borrow().get_parameter_string(addr) });

            (
                iter.next().unwrap(),
                iter.next().unwrap(),
                iter.next().unwrap(),
            )
        };

        OpenGlInfo {
            renderer,
            version,
            shading_language_version,
        }
    }

    pub fn create_program(&mut self) -> Rc<RefCell<Program>> {
        let program = Rc::new(RefCell::new(Program::with_gl(&self.gl)));
        self.programs.push(program.clone());

        program
    }

    pub fn render(&self) {
        let gl = self.gl.borrow();

        unsafe {
            gl.clear_color(1.0, 1.0, 1.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);

            // gl.polygon_mode(
            //     glow::FRONT_AND_BACK,
            //     if wireframe { glow::LINE } else { glow::FILL },
            // );
            for program in &self.programs {
                program.borrow().render();
            }

            // TODO: Count should reflect the current program
            // gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }

    pub fn clear_program(&self) {
        unsafe { self.gl.borrow().use_program(None) };
    }
}

pub enum ShaderType {
    Vertex,
    Fragment,
}
impl From<ShaderType> for u32 {
    fn from(ty: ShaderType) -> Self {
        match ty {
            ShaderType::Vertex => glow::VERTEX_SHADER,
            ShaderType::Fragment => glow::FRAGMENT_SHADER,
        }
    }
}

pub struct Program {
    program: NativeProgram,
    gl: Rc<RefCell<Context>>,
    shaders: Vec<NativeShader>,
    vertices: Vec<(NativeBuffer, NativeVertexArray)>,
    vertex_count: Option<u32>,
    uniform_locations: HashMap<String, UniformLocation>,
}

impl Program {
    pub fn use_program(&self) {
        let gl = self.gl.borrow();
        unsafe { gl.use_program(Some(self.program)) };
    }

    pub fn render(&self) {
        self.use_program();

        let gl = self.gl.borrow();
        if let Some(vertex_count) = self.vertex_count {
            unsafe { gl.draw_arrays(glow::TRIANGLES, 0, vertex_count as i32) };
        }
    }

    pub fn attach_shader(
        &mut self,
        shader_type: ShaderType,
        source: &str,
    ) -> Result<(), OpenGlError> {
        let gl = self.gl.borrow();

        if let Ok(shader) = unsafe { gl.create_shader(shader_type.into()) } {
            unsafe {
                // Comopile shader
                gl.shader_source(shader, source);
                gl.compile_shader(shader);

                if !gl.get_shader_compile_status(shader) {
                    return Err(OpenGlError::ShaderCompile(gl.get_shader_info_log(shader)));
                }

                // Add shader to program
                gl.attach_shader(self.program, shader);
            }

            self.shaders.push(shader);

            Ok(())
        } else {
            Err(OpenGlError::ShaderCreate)
        }
    }

    pub fn attach_vertices(
        &mut self,
        vertices: &[f32],
        format: &[VertexFormat],
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

        // Create the vertex buffer
        let vertex_buffer = unsafe {
            let vertex_buffer = gl.create_buffer().map_err(OpenGlError::BufferCreate)?;

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STATIC_DRAW);

            vertex_buffer
        };

        // Create the vertex array object
        let vertex_array = unsafe {
            let vertex_array = gl
                .create_vertex_array()
                .map_err(OpenGlError::VertexArrayCreate)?;
            gl.bind_vertex_array(Some(vertex_array));

            vertex_array
        };

        let vertex_size = format.iter().fold(0, |size, format| size + format.size());
        let offsets = format.iter().fold(
            {
                let mut v = Vec::with_capacity(format.len());
                v.push(0); // Initial value will have 0 offset
                v
            },
            |mut offsets, format| {
                offsets.push(offsets.last().cloned().unwrap_or(0) + format.size());

                offsets
            },
        );

        for (i, format) in format.iter().enumerate() {
            unsafe {
                gl.enable_vertex_attrib_array(i as u32);
                gl.vertex_attrib_pointer_f32(
                    i as u32,
                    format.count as i32,
                    (&format.vertex_type).into(),
                    false,
                    vertex_size as i32,
                    offsets[i] as i32,
                );
            }
        }

        // TODO: Will change when take other slices
        self.vertex_count = Some(
            (vertices.len() as u32) / format.iter().fold(0, |count, format| count + format.count),
        );
        dbg!(self.vertex_count);

        self.vertices.push((vertex_buffer, vertex_array));

        Ok(())
    }

    pub fn link(&mut self) -> Result<(), OpenGlError> {
        let gl = self.gl.borrow();

        unsafe {
            gl.link_program(self.program);
            if !gl.get_program_link_status(self.program) {
                return Err(OpenGlError::LinkError);
            }
        }

        // Clean up shaders
        for shader in &self.shaders {
            unsafe {
                gl.detach_shader(self.program, *shader);
                gl.delete_shader(*shader);
            }
        }
        self.shaders = Vec::new();

        Ok(())
    }

    pub fn set_uniform(&mut self, name: &str, value: f32) -> Result<(), OpenGlError> {
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

        unsafe { gl.uniform_1_f32(Some(location), value) };

        Ok(())
    }

    fn with_gl(gl: &Rc<RefCell<Context>>) -> Self {
        let gl = gl.clone();
        let program = unsafe { gl.borrow().create_program().expect("program to be created") };

        Self {
            program,
            gl,
            shaders: Vec::new(),
            vertices: Vec::new(),
            vertex_count: None,
            uniform_locations: HashMap::new(),
        }
    }
}

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
