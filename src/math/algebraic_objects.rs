pub mod algebra_r3 {
    use std::ops::{Add, AddAssign, Div, Sub};

    #[derive(Copy, Clone)]
    pub struct Transform {
        pub translation: Vector,
        pub rotation: Quaternion,
        pub scale: Vector
    }

    impl Transform {
        pub fn default() -> Self {
            Self {
                translation: Vector { x: 0., y: 0., z: 0. },
                rotation: Quaternion { r: 1., i: 0., j: 0., k: 0. },
                scale: Vector { x: 1., y: 1., z: 1. }
            }
        }

        fn local_transform<F>(&self, v: Vector, local_origin: Vector, f: F) -> Vector
        where
            F: Fn(Vector) -> Vector
        {
            // move to center, apply f, move back
            f(
                v.translate(local_origin.negate())
            ).translate(local_origin)
        }

        pub fn apply_transform(&self, v: Vector, local_origin: Vector) -> Vector {
            self.local_transform(v, local_origin, |v| {
                Quaternion::from(
                    v.scale(self.scale)
                ).rotate(self.rotation).into()
            }).translate(self.translation)
        }
    }




    #[derive(Copy, Clone)]
    pub struct Quaternion {
        pub r: f32,
        pub i: f32,
        pub j: f32,
        pub k: f32
    }
    impl Quaternion {
        pub fn new_rotation(angle: f32, normal: Vector) -> Quaternion {
            let normal = normal.normalize();
            let s = f32::sin(angle / 2.0);
            Quaternion {
                r: f32::cos(angle / 2.0),
                i: s * normal.x,
                j: s * normal.y,
                k: s * normal.z,
            }
        }
        pub fn conjugate(self) -> Quaternion {
            Quaternion {
                r: self.r,
                i: -self.i,
                j: -self.j,
                k: -self.k
            }
        }
        pub fn multiply(self, p: Quaternion) -> Quaternion {
            Quaternion {
                r: self.r*p.r - self.i*p.i - self.j*p.j - self.k*p.k,
                i: self.r*p.i + self.i*p.r + self.j*p.k - self.k*p.j,
                j: self.r*p.j - self.i*p.k + self.j*p.r + self.k*p.i,
                k: self.r*p.k + self.i*p.j - self.j*p.i + self.k*p.r
            }
        }
        pub fn rotate(self, q: Quaternion) -> Quaternion {
            q.multiply(self).multiply(q.conjugate())
        }
    }
    impl From<Vector> for Quaternion {
        fn from(v: Vector) -> Self {
            Self {
                r: 0.,
                i: v.x,
                j: v.y,
                k: v.z,
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct Vector {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }
    impl Add for Vector {
        type Output = Vector;
        fn add(self, other: Vector) -> Vector {
            self.translate(other)
        }
    }
    impl Sub for Vector {
        type Output = Vector;
        fn sub(self, rhs: Self) -> Self::Output { self.difference(rhs) }
    }
    impl AddAssign for Vector {
        fn add_assign(&mut self, rhs: Self) {
            self.x += rhs.x;
            self.y += rhs.y;
            self.z += rhs.z;
        }
    }
    impl Div<f32> for Vector {
        type Output = Vector;

        fn div(self, rhs: f32) -> Self::Output {
            Vector {
                x: self.x / rhs,
                y: self.y / rhs,
                z: self.z / rhs,
            }
        }
    }
    impl Vector {
        pub fn zero() -> Vector {
            Vector {
                x: 0.,
                y: 0.,
                z: 0.,
            }
        }
        pub fn new(x: f32, y: f32, z: f32) -> Vector {
            Vector {
                x,
                y,
                z
            }
        }
        pub fn id(magnitude: f32) -> Vector {
            Vector {
                x: magnitude,
                y: magnitude,
                z: magnitude,
            }
        }

        pub fn negate(self) -> Vector {
            Vector { x: -self.x, y: -self.y, z: -self.z }
        }
        pub fn translate(self, t: Vector) -> Vector {
            Vector {
                x: self.x + t.x,
                y: self.y + t.y,
                z: self.z + t.z
            }
        }
        pub fn difference(self, t: Vector) -> Vector {
            Self::translate(self, Self::negate(t))
        }
        pub fn scale(self, s: Vector) -> Vector {
            Vector {
                x: self.x * s.x,
                y: self.y * s.y,
                z: self.z * s.z
            }
        }
        pub fn scalar_multiply(self, scale: f32) -> Vector {
            Vector {
                x: self.x * scale,
                y: self.y * scale,
                z: self.z * scale,
            }
        }
        pub fn normalize(self) -> Vector {
            let norm = f32::sqrt((self.x * self.x) + (self.y * self.y) + (self.z * self.z));
            Vector {
                x: self.x / norm,
                y: self.y / norm,
                z: self.z / norm
            }
        }
        pub fn dot(self, q: Vector) -> f32 {
            (self.x * q.x) + (self.y * q.y) + (self.z * q.z)
        }

        pub fn cross(self, q: Vector) -> Vector {
            Vector {
                x: (self.y * q.z) - (self.z * q.y),
                y: (self.z * q.x) - (self.x * q.z),
                z: (self.x * q.y) - (self.y * q.x),
            }
        }
    }
    impl From<Quaternion> for Vector {
        fn from(q: Quaternion) -> Self {
            Self {
                x: q.i,
                y: q.j,
                z: q.k
            }
        }
    }
}