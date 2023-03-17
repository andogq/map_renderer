use glow::{Context, HasContext};
use std::{
    cell::RefCell,
    error::Error,
    fmt::{Debug, Display},
    rc::Rc,
};

mod program;
pub use program::*;

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
        }
    }

    pub fn clear_program(&self) {
        unsafe { self.gl.borrow().use_program(None) };
    }
}
