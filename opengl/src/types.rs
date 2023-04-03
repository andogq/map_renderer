use gl::types::{GLint, GLuint};

macro_rules! gl_wrapper {
    ($struct_name:ident($gl_mapping:path)) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $struct_name(pub(crate) $gl_mapping);
        impl From<$struct_name> for $gl_mapping {
            fn from(value: $struct_name) -> Self {
                value.0
            }
        }
    };
}

gl_wrapper!(Location(GLint));
gl_wrapper!(Program(GLuint));
gl_wrapper!(Shader(GLuint));
gl_wrapper!(Buffer(GLuint));
gl_wrapper!(VertexArrayObject(GLuint));
