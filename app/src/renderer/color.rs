use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<&Color> for raqote::SolidSource {
    fn from(color: &Color) -> Self {
        Self::from_unpremultiplied_argb(0xff, color.r, color.g, color.b)
    }
}

impl From<&Color> for raqote::Source<'_> {
    fn from(color: &Color) -> Self {
        Self::Solid(color.into())
    }
}

impl From<Color> for Vec3 {
    fn from(color: Color) -> Self {
        Vec3::new(
            color.r as f32 / 255_f32,
            color.g as f32 / 255_f32,
            color.b as f32 / 255_f32,
        )
    }
}
