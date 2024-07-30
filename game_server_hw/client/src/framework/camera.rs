use bytemuck;
use cgmath::{InnerSpace, Point3, Vector3, Matrix4};


#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);


pub struct CameraComponent {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

pub trait Camera {
    fn component(&self) -> &CameraComponent;

    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let comp = self.component();
        let view = Matrix4::look_at_rh(comp.eye, comp.target, comp.up);
        let proj = cgmath::perspective(cgmath::Deg(comp.fovy), comp.aspect, comp.znear, comp.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    fn up(&self) -> Vector3<f32> {
        self.component().up
    }

    fn forward(&self) -> Vector3<f32> {
        let comp = self.component();
        let forward = comp.target - comp.eye;
        forward.normalize()
    }

    fn right(&self) -> Vector3<f32> {
        self.forward().cross(self.up())
    }
}


pub struct DefaultCamera {
    pub component: CameraComponent,
}

impl Camera for DefaultCamera {
    fn component(&self) -> &CameraComponent {
        &self.component
    }
}

impl From<CameraComponent> for DefaultCamera {
    fn from(component: CameraComponent) -> Self {
        Self { component }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;

        Self {
            view_proj: cgmath::Matrix4::identity().into()
        }
    }

    pub fn update_view_proj(&mut self, camera_view_proj: cgmath::Matrix4<f32>) {
        self.view_proj = camera_view_proj.into();
    }
}
