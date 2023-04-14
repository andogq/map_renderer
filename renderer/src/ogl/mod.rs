mod program;
pub mod texture_buffer;

use glam::Vec4;
use opengl::{BufferMask, Context, StringName};
pub use program::*;
use std::{
    cell::RefCell,
    error::Error,
    fmt::{Debug, Display},
    rc::Rc,
};

use self::texture_buffer::{TextureBuffer, TextureBufferBuilder, TextureBufferBuilderError};

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
                StringName::Renderer,
                StringName::Version,
                StringName::ShaderLanguageVersion,
            ]
            .into_iter()
            .map(|string_name| self.gl.borrow().get_string(string_name));

            (
                iter.next().unwrap().unwrap(),
                iter.next().unwrap().unwrap(),
                iter.next().unwrap().unwrap(),
            )
        };

        OpenGlInfo {
            renderer,
            version,
            shading_language_version,
        }
    }

    pub fn add_program(
        &mut self,
        builder: ProgramBuilder,
    ) -> Result<Rc<RefCell<Program>>, ProgramBuilderError> {
        // Build the program
        let program = builder.with_gl(self.gl.clone()).build()?;

        // Wrap it with Rc<Ref<T>>
        let program = Rc::new(RefCell::new(program));

        // Save the program
        self.programs.push(program.clone());

        Ok(program)
    }

    pub fn render(&self) {
        let gl = self.gl.borrow();

        gl.clear_color(Vec4::new(1.0, 1.0, 1.0, 1.0));
        gl.clear(BufferMask::Color);

        // gl.polygon_mode(
        //     glow::FRONT_AND_BACK,
        //     if wireframe { glow::LINE } else { glow::FILL },
        // );
        for program in &self.programs {
            program.borrow().render();
        }
    }

    pub fn create_texture(
        &self,
        builder: TextureBufferBuilder,
    ) -> Result<TextureBuffer, TextureBufferBuilderError> {
        builder.build(self.gl.clone())
    }

    pub fn clear_program(&self) {
        self.gl.borrow().clear_program();
    }
}
