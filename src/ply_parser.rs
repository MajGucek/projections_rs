use std::fs::File;
use std::io::BufReader;
use std::ops::{AddAssign, Sub};
use ply_rs::parser::Parser;
use ply_rs::ply::{DefaultElement, Ply, PropertyAccess};
use crate::math::algebra_r3::{Transform, Vector};
use crate::math::math_lib::{MeshData, Shape, TriFace, RGB};

pub struct PlyObject {
    mesh_data: MeshData,

    transform: Transform,
    #[allow(unused)] // May come in handy?
    file_name: String,
}



impl PlyObject {
    pub fn try_load(file_name: String) -> Result<Self, String> {
        let f = File::open(&file_name).map_err(|err| format!("{:?}", err))?;
        let mut f = BufReader::new(f);

        let parser = Parser::<DefaultElement>::new();
        let ply: Ply<DefaultElement> = parser.read_ply(&mut f).map_err(|err| format!("{:?}", err))?;

        let vertex_list = ply.payload.get("vertex").ok_or("no vertex field in payload!")?;
        let mut vertices = Vec::with_capacity(vertex_list.len());
        for vertex in vertex_list {
            let x = vertex.get_float(&"x".to_owned()).ok_or("No x found!")?;
            let y = vertex.get_float(&"y".to_owned()).ok_or("No y found!")?;
            let z = vertex.get_float(&"z".to_owned()).ok_or("No z found!")?;
            vertices.push(Vector { x, y, z} );
        }
        let vertex_count = vertices.len();

        let face_list = ply.payload.get("face").ok_or("no face field in payload")?;
        let mut face_indices = Vec::with_capacity(face_list.len());
        for face in face_list {
            let indices = face.get_list_int(&"vertex_indices".to_owned()).ok_or("No vertex_indices found!")?;
            if indices.len() == 3 {
                face_indices.push(
                    [indices[0] as usize, indices[1] as usize, indices[2] as usize]
                );
            }
        }

        let mut normals = vec![Vector::zero(); vertex_count];
        for &[i0, i1, i2] in &face_indices {
            let v0 = vertices[i0];
            let v1 = vertices[i1];
            let v2 = vertices[i2];

            let face_normal = (v1 - v0).cross(v2 - v0).normalize();

            normals[i0] += face_normal;
            normals[i1] += face_normal;
            normals[i2] += face_normal;
        }

        for n in &mut normals {
            *n = n.normalize();
        }

        let local_origin = Vector {
            x: 1.0 / (vertex_count as f32) * vertices.iter().fold(0., |acc, vec| acc + vec.x),
            y: 1.0 / (vertex_count as f32) * vertices.iter().fold(0., |acc, vec| acc + vec.y),
            z: 1.0 / (vertex_count as f32) * vertices.iter().fold(0., |acc, vec| acc + vec.z),
        };
        
        Ok(Self {
            mesh_data: MeshData {
                vertices,
                normals,
                face_indices,
                local_origin,
                base_color: RGB::default(),
            },
            transform: Transform::default(),
            file_name
        })
    }
}


impl Shape for PlyObject {
    fn get_color(&self) -> &RGB {
        &self.mesh_data.base_color
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }
    fn get_mesh_data(&self) -> &MeshData { &self.mesh_data }

    fn get_local_origin(&self) -> &Vector { &self.mesh_data.local_origin }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }


}