use super::color::*;


pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    // pub tex_coords: [f32; 2],
    pub base_color: Color,
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ]
        }
    }
}



pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    // pub material: usize,
}

use super::object::*;

use std::{
    rc::Rc, 
    cell::RefCell, 
};

pub struct Model {
    pub meshes: Vec<Mesh>,
    // pub materials: Vec<Material>,
    pub buffer: wgpu::Buffer, 
    pub instances: Vec<Rc<RefCell<Object>>>,
}

impl Model {
    pub async fn load(
        file_name: &str,
        device: &wgpu::Device,
        scale_factor: f32,
        base_color: Color,
    ) -> anyhow::Result<Model> {
        use std::{mem, io::{BufReader, Cursor}};
        use wgpu::util::DeviceExt;
        use super::transform::*;
        use super::resources::*;

        let obj_text = load_string(file_name).await?;
        let obj_cursor = Cursor::new(obj_text);
        let mut obj_reader = BufReader::new(obj_cursor);
    
        let (models, _obj_materials) = tobj::load_obj_buf_async(
            &mut obj_reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| async move {
                let mat_text = load_string(&p).await.unwrap();
                tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
            },
        )
        .await?;
    
        let buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Instance Buffer"), 
                mapped_at_creation: false, 
                size: (mem::size_of::<TransformRaw>() * 128) as wgpu::BufferAddress, 
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, 
            }
        );

        let meshes = models
            .into_iter()
            .map(|m| {
                    let vertices = (0..m.mesh.positions.len() / 3)
                    .map(|i| {
                        if m.mesh.normals.is_empty() {
                            ModelVertex {
                                position: [
                                    m.mesh.positions[i * 3] * scale_factor,
                                    m.mesh.positions[i * 3 + 1] * scale_factor,
                                    m.mesh.positions[i * 3 + 2] * scale_factor,
                                ],
                                base_color,
                                // tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                                normal: [0.0, 0.0, 0.0],
                            }
                        }
                        else {
                            ModelVertex {
                                position: [
                                    m.mesh.positions[i * 3] * scale_factor,
                                    m.mesh.positions[i * 3 + 1] * scale_factor,
                                    m.mesh.positions[i * 3 + 2] * scale_factor,
                                ],
                                // tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                                base_color,
                                normal: [
                                    m.mesh.normals[i * 3],
                                    m.mesh.normals[i * 3 + 1],
                                    m.mesh.normals[i * 3 + 2],
                                ],
                            }
                        }
                    })
                    .collect::<Vec<_>>();
    
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", file_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", file_name)),
                    contents: bytemuck::cast_slice(&m.mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
    
                Mesh {
                    name: file_name.to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: m.mesh.indices.len() as u32,
                    // material: m.mesh.material_id.unwrap_or(0),
                }
            })
            .collect::<Vec<_>>();
    
        Ok(Model { meshes, buffer, instances: Vec::with_capacity(128) })
    }

    pub fn add_instance(&mut self, object: Rc<RefCell<Object>>) {
        self.instances.push(object);
    }

    pub fn remove_instance(&mut self, object: Rc<RefCell<Object>>) {
        self.instances.retain(|obj| obj.as_ptr() != object.as_ptr());
    }

    pub fn draw<'a>(&'a self, queue: &wgpu::Queue, rpass: &mut wgpu::RenderPass<'a>) {
        let data: Vec<_> = self.instances.iter()
            .map(|instance| instance.borrow().transform.to_raw())
            .take(128)
            .collect();

        queue.write_buffer(
            &self.buffer, 
            0, 
            bytemuck::cast_slice(&data)
        );

        rpass.set_vertex_buffer(1, self.buffer.slice(..));
        for mesh in self.meshes.iter() {
            rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            rpass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..mesh.num_elements, 0, 0..self.instances.len() as u32);
        }
    }
}
