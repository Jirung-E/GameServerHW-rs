#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color(f32, f32, f32, f32);

impl Color {
    pub const RED: Color = Color(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Color = Color(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Color = Color(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Color = Color(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Color = Color(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color(1.0, 0.0, 1.0, 1.0);
    pub const WHITE: Color = Color(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Color = Color(0.0, 0.0, 0.0, 1.0);

    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        let r = r.clamp(0.0, 1.0);
        let g = g.clamp(0.0, 1.0);
        let b = b.clamp(0.0, 1.0);
        let a = a.clamp(0.0, 1.0);
        
        Self(r, g, b, a)
    }
}