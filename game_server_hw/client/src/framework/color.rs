#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color(f32, f32, f32, f32);

impl Color {
    pub const RED: Color        = Color(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Color      = Color(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Color       = Color(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Color     = Color(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Color       = Color(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Color    = Color(1.0, 0.0, 1.0, 1.0);
    pub const WHITE: Color      = Color(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Color      = Color(0.0, 0.0, 0.0, 1.0);
    pub const GRAY: Color       = Color(0.5, 0.5, 0.5, 1.0);
    pub const LIGHT_GRAY: Color = Color(0.75, 0.75, 0.75, 1.0);
    pub const DARK_GRAY: Color  = Color(0.25, 0.25, 0.25, 1.0);

    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        let r = r.clamp(0.0, 1.0);
        let g = g.clamp(0.0, 1.0);
        let b = b.clamp(0.0, 1.0);
        let a = a.clamp(0.0, 1.0);
        
        Self(r, g, b, a)
    }

    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self::from_rgba(r, g, b, 1.0)
    }

    pub fn r(&self) -> f32 {
        self.0
    }

    pub fn g(&self) -> f32 {
        self.1
    }

    pub fn b(&self) -> f32 {
        self.2
    }

    pub fn a(&self) -> f32 {
        self.3
    }
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self::from_rgba(arr[0], arr[1], arr[2], arr[3])
    }
}

impl From<[f32; 3]> for Color {
    fn from(arr: [f32; 3]) -> Self {
        Self::from_rgba(arr[0], arr[1], arr[2], 1.0)
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.0, self.1, self.2, self.3]
    }
}

impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        wgpu::Color {
            r: self.0 as f64,
            g: self.1 as f64,
            b: self.2 as f64,
            a: self.3 as f64,
        }
    }
}