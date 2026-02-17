pub mod math_lib {
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
        fn from(value: Vector) -> RGB {
            RGB {
                r: (value.x * 255.) as u8,
                g: (value.y * 255.) as u8,
                b: (value.z * 255.) as u8,
            }
        }
    }
    impl Into<Vector> for RGB {
        fn into(self) -> Vector {
            Vector {
                x: self.r as f32 / 255.,
                y: self.g as f32 / 255.,
                z: self.b as f32 / 255.,
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
        pub normal: Vector,
    }
    impl TriFace {
        pub fn calculate_colors(&mut self, light: &Light, ambient_light: f32) {

            for i in 0..3 {
                let base = self.colors[i];

                let base_vec: Vector = base.into();

                let diffuse_intensity = self.normal.dot(light.direction).max(0.0);
                let diffuse = base_vec * diffuse_intensity * light.intensity;
                let ambient = base_vec * ambient_light;

                let final_color_vec = diffuse + ambient;
                let final_color = Vector {
                    x: final_color_vec.x.clamp(0.0, 1.0),
                    y: final_color_vec.y.clamp(0.0, 1.0),
                    z: final_color_vec.z.clamp(0.0, 1.0),
                };

                self.colors[i] = final_color.into();
            }
        }

        pub fn project_to_screen_space(&self, focal_length: f32, x_offset: f32, y_offset: f32, zoom: f32) -> (egui::Mesh, f32) {
            let mut mesh = egui::Mesh::default();
            let mut depth_avg = 0.;
            for i in 0..3 {
                let (pos2, depth) = project(focal_length, x_offset, y_offset, zoom, &self.vertices[i]);
                
                depth_avg += depth;

                mesh.vertices.push(egui::epaint::Vertex {
                    pos: pos2,
                    uv: egui::pos2(0., 0.),
                    color: self.colors[i].into()
                });
            }
            mesh.indices.extend([0, 1, 2]);
            
            depth_avg /= 3.;
            (mesh, depth_avg)    
        }


        pub fn check_face_culling(&self, focal_length: f32) -> bool {
            check_face_culling(focal_length, &self)
        }
    }
    impl From<([Vector; 3], RGB)> for TriFace {
        fn from(value: ([Vector; 3], RGB)) -> Self {
            Self {
                vertices: [value.0[0], value.0[1], value.0[2]],
                colors: [value.1, value.1, value.1],
                normal: value.0[1].difference(value.0[0]).cross(
                    value.0[2].difference(value.0[0])
                ).normalize()
            }
        }
    }


    pub struct Light {
        pub direction: Vector,
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
        fn get_vertices(&self) -> &[Vector];
        fn get_transform(&self) -> &Transform;
        fn set_transform(&mut self, transform: Transform);

        fn transform_vectors(&self, scale: f32) -> Vec<Vector> {
            self.get_vertices()
                .iter()
                .copied()
                .map(|vertex| {
                    self.get_transform().apply_transform(vertex, self.local_origin()) * scale
                })
                .collect()
        }

        fn produce_mesh(&self) -> Vec<TriFace>;

        fn local_origin(&self) -> Vector {
            Vector {
                x: 1.0 / (self.get_vertices().len() as f32) * self.get_vertices().iter().fold(0., |acc, vec| acc + vec.x),
                y: 1.0 / (self.get_vertices().len() as f32) * self.get_vertices().iter().fold(0., |acc, vec| acc + vec.y),
                z: 1.0 / (self.get_vertices().len() as f32) * self.get_vertices().iter().fold(0., |acc, vec| acc + vec.z),
            }
        }
    }


}