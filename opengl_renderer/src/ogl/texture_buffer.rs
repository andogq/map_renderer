use opengl::{
    Buffer, BufferType, Context, ImageFormat, OpenGlError, Texture, TextureTarget, Usage,
};
use std::{cell::RefCell, error::Error, fmt::Display, rc::Rc};

#[derive(Debug)]
pub enum TextureBufferBuilderError {
    MissingFormat,
    OpenGlError(OpenGlError),
}
impl Display for TextureBufferBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingFormat => write!(f, "Missing format field"),
            Self::OpenGlError(e) => e.fmt(f),
        }
    }
}
impl Error for TextureBufferBuilderError {}

#[derive(Default)]
pub struct TextureBufferBuilder {
    format: Option<ImageFormat>,
}

impl TextureBufferBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_format(mut self, format: ImageFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn build(
        self,
        gl: Rc<RefCell<Context>>,
    ) -> Result<TextureBuffer, TextureBufferBuilderError> {
        let (texture, buffer) = {
            let gl = gl.borrow();
            let texture = gl
                .create_texture()
                .map_err(TextureBufferBuilderError::OpenGlError)?;
            let buffer = gl
                .create_buffer()
                .map_err(TextureBufferBuilderError::OpenGlError)?;

            (texture, buffer)
        };

        Ok(TextureBuffer {
            gl,
            texture,
            buffer,
            format: self
                .format
                .ok_or(TextureBufferBuilderError::MissingFormat)?,
        })
    }
}

pub struct TextureBuffer {
    gl: Rc<RefCell<Context>>,
    texture: Texture,
    buffer: Buffer,
    format: ImageFormat,
}

impl TextureBuffer {
    pub fn bind(&self, texture_number: u32) {
        let gl = self.gl.borrow();

        // Activate the texture
        gl.active_texture(texture_number);
        gl.bind_texture(TextureTarget::Buffer, self.texture);
        gl.texture_buffer(self.format, self.buffer);
    }

    pub fn set_data(&self, data: &[u8]) {
        let gl = self.gl.borrow();

        gl.bind_buffer(BufferType::TextureBuffer, self.buffer);
        gl.buffer_data_u8_slice(BufferType::TextureBuffer, data, Usage::StaticDraw);
    }
}
