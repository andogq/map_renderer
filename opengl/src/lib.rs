mod enums;
mod types;

pub use enums::*;
use gl::types::GLuint;
pub use types::*;

use glam::{Mat4, Vec4};
use std::error::Error;
use std::ffi::{self, c_void, CString};
use std::ffi::{c_char, CStr};
use std::fmt::Display;
use std::ptr;

#[derive(Debug)]
pub enum OpenGlError {
    ShaderCreation,
    GetString,
    VertexArrayCreation,
}
impl Display for OpenGlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShaderCreation => write!(f, "Problem creating a shader"),
            Self::GetString => write!(f, "Problem getting a string"),
            Self::VertexArrayCreation => write!(f, "Probem creating vertex array"),
        }
    }
}
impl Error for OpenGlError {}

type Result<T> = std::result::Result<T, OpenGlError>;

pub struct Context(());
impl Context {
    pub fn load<F>(loader: F) -> Self
    where
        F: Fn(&CStr) -> *const ffi::c_void,
    {
        gl::load_with(|s| loader(&CString::new(s).unwrap()));

        Context(())
    }
}

impl Context {
    // Meta functions
    pub fn get_string(&self, name: StringName) -> Result<String> {
        let ptr = unsafe { gl::GetString(name.into()) };

        if ptr.is_null() {
            Err(OpenGlError::GetString)
        } else {
            let cstr = unsafe { CStr::from_ptr(ptr as *const c_char) };

            Ok(String::from_utf8_lossy(cstr.to_bytes()).to_string())
        }
    }

    pub fn enable(&self, capability: Capability) {
        unsafe { gl::Enable(capability.into()) };
    }

    pub fn draw_arrays(&self, draw_type: DrawType, offset: u32, count: u32) {
        unsafe { gl::DrawArrays(draw_type.into(), offset as i32, count as i32) };
    }

    // Object creation
    pub fn create_program(&self) -> Result<Program> {
        match unsafe { gl::CreateProgram() } {
            0 => Err(OpenGlError::ShaderCreation),
            program => Ok(Program(program)),
        }
    }

    pub fn create_shader(&self, r#type: ShaderType) -> Result<Shader> {
        match unsafe { gl::CreateShader(r#type.into()) } {
            0 => Err(OpenGlError::ShaderCreation),
            shader => Ok(Shader(shader)),
        }
    }

    pub fn create_buffer(&self) -> Result<Buffer> {
        let mut buffer: GLuint = 0;
        unsafe { gl::GenBuffers(1, &mut buffer as *mut GLuint) };

        match buffer {
            0 => Err(OpenGlError::ShaderCreation),
            buffer => Ok(Buffer(buffer)),
        }
    }

    pub fn create_vertex_array(&self) -> Result<VertexArrayObject> {
        let mut array: GLuint = 0;
        unsafe { gl::GenVertexArrays(1, &mut array as *mut GLuint) };

        match array {
            0 => Err(OpenGlError::VertexArrayCreation),
            array => Ok(VertexArrayObject(array)),
        }
    }

    pub fn create_texture(&self) -> Result<Texture> {
        let mut texture: GLuint = 0;
        unsafe { gl::GenTextures(1, &mut texture as *mut GLuint) };

        match texture {
            0 => Err(OpenGlError::VertexArrayCreation),
            texture => Ok(Texture(texture)),
        }
    }

    // (Color) Buffer handling
    pub fn clear_color(&self, color: Vec4) {
        unsafe { gl::ClearColor(color.x, color.y, color.z, color.w) }
    }

    pub fn clear(&self, buffer: BufferMask) {
        unsafe { gl::Clear(buffer.into()) };
    }

    // Uniforms
    pub fn get_uniform_location(&self, program: Program, name: &str) -> Option<Location> {
        let name = CString::new(name).unwrap();
        match unsafe { gl::GetUniformLocation(program.into(), name.as_ptr().cast()) } {
            -1 => None,
            location => Some(Location(location)),
        }
    }

    pub fn uniform_f32(&self, location: Location, value: f32) {
        unsafe { gl::Uniform1f(location.into(), value) };
    }

    pub fn uniform_mat4(&self, location: Location, value: &Mat4) {
        unsafe {
            gl::UniformMatrix4fv(
                location.into(),
                1,
                false.into(),
                value.to_cols_array().as_ptr(),
            )
        };
    }

    pub fn uniform_i32(&self, location: Location, value: i32) {
        unsafe { gl::Uniform1i(location.into(), value) };
    }

    // Shaders
    pub fn shader_source(&self, shader: Shader, source: &str) {
        let source = CString::new(source).unwrap();

        unsafe { gl::ShaderSource(shader.into(), 1, &source.as_ptr().cast(), ptr::null()) };
    }

    pub fn compile_shader(&self, shader: Shader) {
        unsafe { gl::CompileShader(shader.into()) };
    }

    pub fn attach_shader(&self, program: Program, shader: Shader) {
        unsafe { gl::AttachShader(program.into(), shader.into()) };
    }

    pub fn detach_shader(&self, program: Program, shader: Shader) {
        unsafe { gl::DetachShader(program.into(), shader.into()) };
    }

    pub fn delete_shader(&self, shader: Shader) {
        unsafe { gl::DeleteShader(shader.into()) };
    }

    // Utility for other parameter functions
    unsafe fn get_shader_parameter(&self, shader: Shader, parameter: ShaderParameter) -> i32 {
        let mut value = 0i32;
        gl::GetShaderiv(shader.into(), parameter.into(), &mut value as *mut i32);
        value
    }

    pub fn get_shader_compile_status(&self, shader: Shader) -> bool {
        unsafe { self.get_shader_parameter(shader, ShaderParameter::CompileStatus) != 0 }
    }

    pub fn get_shader_info_log(&self, shader: Shader) -> String {
        // Get the size of the shader info log
        let size =
            dbg!(unsafe { self.get_shader_parameter(shader, ShaderParameter::InfoLogLength) });
        let mut buffer: Vec<u8> = vec![0; size as usize];

        unsafe {
            gl::GetShaderInfoLog(
                shader.into(),
                size + 10,
                ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
            )
        };

        let str = CString::from_vec_with_nul(buffer).unwrap();

        String::from_utf8_lossy(str.as_bytes()).to_string()
    }

    // Programs
    pub fn clear_program(&self) {
        unsafe { gl::UseProgram(0) };
    }

    pub fn link_program(&self, program: Program) {
        unsafe { gl::LinkProgram(program.into()) };
    }

    unsafe fn get_program_parameter(&self, program: Program, parameter: ProgramParameter) -> i32 {
        let mut value = 0i32;
        gl::GetProgramiv(program.into(), parameter.into(), &mut value as *mut i32);
        value
    }

    pub fn get_program_link_status(&self, program: Program) -> bool {
        unsafe { self.get_program_parameter(program, ProgramParameter::LinkStatus) != 0 }
    }

    pub fn get_program_info_log(&self, program: Program) -> String {
        // Get the size of the shader info log
        let size = unsafe { self.get_program_parameter(program, ProgramParameter::InfoLogLength) };
        let mut buffer: Vec<u8> = vec![0; size as usize];

        unsafe {
            gl::GetProgramInfoLog(
                program.into(),
                size,
                ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
            )
        };

        let str = CString::from_vec_with_nul(buffer).unwrap();

        String::from_utf8_lossy(str.as_bytes()).to_string()
    }

    pub fn use_program(&self, program: Program) {
        unsafe { gl::UseProgram(program.into()) };
    }

    // Buffers
    pub fn bind_buffer(&self, target: BufferType, buffer: Buffer) {
        unsafe { gl::BindBuffer(target.into(), buffer.into()) };
    }

    // Attributes
    pub fn enable_vertex_attribute_array(&self, index: u32) {
        unsafe { gl::EnableVertexAttribArray(index) };
    }

    pub fn vertex_attribute_pointer_f32(
        &self,
        index: u32,
        count: u32,
        r#type: DataType,
        normalized: bool,
        stride: u32,
        offset: u32,
    ) {
        unsafe {
            gl::VertexAttribPointer(
                index,
                count as i32,
                r#type.into(),
                normalized as u8,
                stride as i32,
                offset as *const c_void,
            )
        };
    }

    pub fn buffer_data_u8_slice(&self, target: BufferType, data: &[u8], usage: Usage) {
        unsafe {
            gl::BufferData(
                target.into(),
                data.len() as isize,
                data.as_ptr() as *const c_void,
                usage.into(),
            )
        };
    }

    // Vertex Array
    pub fn bind_vertex_array(&self, vertex_array: VertexArrayObject) {
        unsafe { gl::BindVertexArray(vertex_array.into()) };
    }

    // Textures
    pub fn texture_buffer(&self, internal_format: ImageFormat, buffer: Buffer) {
        unsafe {
            gl::TexBuffer(
                BufferType::TextureBuffer.into(),
                internal_format.into(),
                buffer.into(),
            )
        };
    }

    pub fn active_texture(&self, texture_i: u32) {
        unsafe { gl::ActiveTexture(gl::TEXTURE0 + texture_i) };
    }

    pub fn get_texture_offset(&self) -> u32 {
        gl::TEXTURE0
    }

    pub fn bind_texture(&self, target: TextureTarget, texture: Texture) {
        unsafe { gl::BindTexture(target.into(), texture.into()) };
    }
}
