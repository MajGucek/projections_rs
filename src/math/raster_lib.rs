pub mod math_lib {
    use std::ops::Div;
    use egui::Pos2;
    use crate::math::algebraic_objects::algebra_r3::*;
    
    fn check_vector_culling(focal_length: f32, v: &Vector) -> bool { v.z + focal_length > 1.0 }
    fn check_face_culling(focal_length: f32, face: &TriFace) -> bool {

        face.vertices.iter().any(|vector| check_vector_culling(focal_length, vector))
    }

    fn project(focal_length: f32, x_offset: f32, y_offset: f32, zoom: f32, v: &Vector) -> (Pos2, f32) {
        (
            Pos2::new(
                (v.x * (focal_length / (v.z + focal_length)) * zoom) + x_offset,
                (v.y * (focal_length / (v.z + focal_length)) * zoom) + y_offset
            ),
            v.z,
        )
    }

    #[derive(Copy, Clone)]
    pub struct RGB {
        pub r: u8,
        pub g: u8,
        pub b: u8
    }
    impl RGB {
        pub fn new(r: u8, g: u8, b: u8) -> Self {
            RGB {
                r,
                g,
                b
            }
        }
        pub fn default() -> Self {
            RGB {
                r: 255,
                g: 255,
                b: 255,
            }
        }
        pub fn add(self, other: RGB) -> Self {
            RGB {
                r: (self.r as u32 + other.r as u32).min(255) as u8,
                g: (self.g as u32 + other.g as u32).min(255) as u8,
                b: (self.b as u32 + other.b as u32).min(255) as u8,
            }
        }
        pub fn scale(self, s: f32) -> Self {
            RGB {
                r: (self.r as f32 * s).min(255.0).max(0.0) as u8,
                g: (self.g as f32 * s).min(255.0).max(0.0) as u8,
                b: (self.b as f32 * s).min(255.0).max(0.0) as u8,
            }
        }

    }
    impl From<Vector> for RGB {
        fn from(value: Vector) -> Self {
            RGB {
                r: (value.x.clamp(0., 1.) * 255.) as u8,
                g: (value.y.clamp(0., 1.) * 255.) as u8,
                b: (value.z.clamp(0., 1.) * 255.) as u8,
            }
        }
    }
    impl From<RGB> for Vector {
        fn from(value: RGB) -> Self {
            Vector {
                x: (value.r as f32) / 255.,
                y: (value.g as f32) / 255.,
                z: (value.b as f32) / 255.,
            }
        }
    }
    impl From<RGB> for egui::Color32 {
        fn from(value: RGB) -> Self {
            egui::Color32::from_rgb(value.r, value.g, value.b)
        }
    }



    #[derive(Copy, Clone)]
    pub struct TriFace {
        pub vertices: [Vector; 3],
        pub colors: [RGB; 3],
        pub normals: [Vector; 3],
    }
    impl TriFace {
        pub fn calculate_colors(&mut self, light: &Light, ambient_light: f32) {

            for i in 0..3 {
                let base = self.colors[i];
                let base_vec: Vector = Vector::from(base);

                let light_dir = light.direction.negate().normalize();
                let diffuse_intensity = self.normals[i].dot(light_dir).max(0.);

                let diffuse = base_vec.scalar_multiply(diffuse_intensity * light.intensity);
                let ambient = base_vec.scalar_multiply(ambient_light);

                let final_color_vec = diffuse + ambient;

                let final_color = Vector {
                    x: final_color_vec.x.clamp(0.0, 1.0),
                    y: final_color_vec.y.clamp(0.0, 1.0),
                    z: final_color_vec.z.clamp(0.0, 1.0),
                };

                self.colors[i] = final_color.into();
            }
        }

        pub fn project_to_screen_space(&self, focal_length: f32, x_offset: f32, y_offset: f32, zoom: f32) -> ProjectedTriangle {
            let mut depth_sum = 0.;

            let vertices = core::array::from_fn(|i| {
                let (pos, depth) = project(
                    focal_length,
                    x_offset,
                    y_offset,
                    zoom,
                    &self.vertices[i],
                );
                depth_sum += depth;
                egui::epaint::Vertex {
                    pos,
                    uv: egui::pos2(0., 0.),
                    color: self.colors[i].into(),
                }
            });

            ProjectedTriangle {
                depth: depth_sum / 3.,
                vertices,
            }
        }


        pub fn check_face_culling(&self, focal_length: f32) -> bool {
            check_face_culling(focal_length, &self)
        }
    }


    pub struct ProjectedTriangle {
        pub depth: f32,
        pub vertices: [egui::epaint::Vertex; 3],
    }

    pub struct MeshData {
        pub vertices: Vec<Vector>, // Virtual space vertex positions
        pub normals: Vec<Vector>, // per vertex normals
        pub face_indices: Vec<[usize; 3]>, // triangle indexes
        pub local_origin: Vector,
        pub base_color: RGB,
    }

    pub struct Light {
        pub direction: Vector,
        #[allow(unused)] // Kind of novelty, pointless
        pub color: RGB,
        pub intensity: f32,
    }
    impl Light {
        pub fn new(direction: Vector, intensity: f32) -> Self {
            Self {
                direction: direction.normalize(),
                color: RGB::default(),
                intensity
            }
        }
    }



    pub trait Shape {
        #[allow(unused)]
        fn get_color(&self) -> &RGB;
        fn get_transform(&self) -> &Transform;
        fn get_mesh_data(&self) -> &MeshData;
        fn get_local_origin(&self) -> &Vector;
        fn set_transform(&mut self, transform: Transform);

        fn iter_triangles_transformed(&self, scale: f32) -> Box<dyn Iterator<Item = TriFace> + '_ > {
            let mesh = self.get_mesh_data();
            let transform = self.get_transform();
            let origin = self.get_local_origin();


            let transformed_vertices: Vec<Vector> = mesh.vertices
                .iter()
                .map(|v| transform.apply_transform(*v, *origin).scalar_multiply(scale))
                .collect();

            let transformed_normals: Vec<Vector> = mesh.normals
                .iter()
                .map(|normal| {
                    transform.apply_transform(*normal, *origin).normalize()
                })
                .collect();


            Box::new(mesh.face_indices.iter().map(move |&[i0, i1, i2]| {
                TriFace {
                    vertices: [
                        transformed_vertices[i0],
                        transformed_vertices[i1],
                        transformed_vertices[i2],
                    ],
                    colors: [*self.get_color(); 3],
                    normals: [
                        transformed_normals[i0],
                        transformed_normals[i1],
                        transformed_normals[i2],
                    ],
                }
            }))

        }


    }


}