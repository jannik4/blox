use bevy_color::prelude::*;
use bevy_math::prelude::*;
use std::{
    f32::consts::PI,
    ops::{Add, AddAssign, Div, Mul},
    thread,
};

pub trait Scene {
    fn lights(&self) -> &[Light];
    fn cast_ray(&self, ray: Ray3d, max_distance: f32) -> Option<RayHit>;
}

#[derive(Debug, Clone, Copy)]
pub enum Light {
    Ambient {
        color: LinearRgb,
        intensity: f32,
    },
    Directional {
        direction: Dir3,
        color: LinearRgb,
        intensity: f32,
    },
    Point {
        position: Vec3,
        color: LinearRgb,
        intensity: f32,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    pub material: Material,
    pub position: Vec3,
    pub normal: Dir3,
    pub distance: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum Material {
    Diffuse {
        albedo: LinearRgb,
    },
    Reflective {
        albedo: LinearRgb,
        reflectivity: f32,
    },
    Refractive {
        albedo: LinearRgb,
        index: f32,
        transparency: f32,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub translation: Vec3,
    pub direction: Dir3,
    pub up: Dir3,
    pub fov: f32,
    pub background: LinearRgb,
}

#[derive(Debug)]
pub struct Renderer {
    camera: Camera,
    dimensions: UVec2,

    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    top_left_pixel: Vec3,

    shadow_bias: f32,
    max_recursion_depth: u32,
}

impl Renderer {
    pub fn init(camera: Camera, dimensions: UVec2) -> Self {
        let focal_length = 1.0;
        let h = f32::tan(camera.fov / 2.0);
        let viewport_height = 2.0 * h * focal_length;
        let viewport_width = viewport_height * (dimensions.x as f32 / dimensions.y as f32);

        let w = -*camera.direction;
        let u = camera.up.cross(w).normalize();
        let v = w.cross(u);

        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        let pixel_delta_u = viewport_u / dimensions.x as f32;
        let pixel_delta_v = viewport_v / dimensions.y as f32;

        let top_left_pixel =
            camera.translation - focal_length * w - viewport_u / 2.0 - viewport_v / 2.0
                + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            camera,
            dimensions,

            pixel_delta_u,
            pixel_delta_v,
            top_left_pixel,

            shadow_bias: 0.001,
            max_recursion_depth: 10,
        }
    }

    pub fn render<S: Scene + Send + Sync>(&self, scene: &S) -> Vec<Color> {
        let mut pixels = vec![Color::BLACK; (self.dimensions.x * self.dimensions.y) as usize];
        self.render_into(scene, &mut pixels);
        pixels
    }

    // TODO: Do in parallel on non-wasm targets
    pub fn render_into<S: Scene + Send + Sync>(&self, scene: &S, pixels: &mut [Color]) {
        assert!(pixels.len() == (self.dimensions.x * self.dimensions.y) as usize);

        // for y in 0..self.dimensions.y {
        //     let offset = (y * self.dimensions.x) as usize;
        //     for x in 0..self.dimensions.x {
        //         let pixel = self.render_pixel(scene, UVec2::new(x, y));
        //         pixels[offset + x as usize] = pixel;
        //     }
        // }

        let threads = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let chunk_size = pixels.len() / threads;
        thread::scope(|s| {
            for (chunk_index, chunk) in pixels.chunks_mut(chunk_size).enumerate() {
                s.spawn(move || {
                    let offset = chunk_index * chunk_size;
                    for (i, pixel) in chunk.iter_mut().enumerate() {
                        let index = offset + i;
                        let x = (index % (self.dimensions.x as usize)) as u32;
                        let y = (index / (self.dimensions.x as usize)) as u32;
                        *pixel = self.render_pixel(scene, UVec2::new(x, y));
                    }
                });
            }
        });
    }

    pub fn render_pixel<S: Scene>(&self, scene: &S, pixel: UVec2) -> Color {
        let pixel = self.top_left_pixel
            + (pixel.x as f32) * self.pixel_delta_u
            + (pixel.y as f32) * self.pixel_delta_v;
        let ray = Ray3d {
            origin: self.camera.translation,
            direction: Dir3::new(pixel - self.camera.translation).unwrap(),
        };

        self.cast_ray(scene, ray, 0).into()
    }

    fn cast_ray<S: Scene>(&self, scene: &S, ray: Ray3d, depth: u32) -> LinearRgb {
        if depth >= self.max_recursion_depth {
            return self.camera.background;
        }

        let Some(surface) = scene.cast_ray(ray, f32::INFINITY) else {
            return self.camera.background;
        };

        match surface.material {
            Material::Diffuse { albedo } => {
                self.shade_diffuse(scene, albedo, surface.position, surface.normal)
            }
            Material::Reflective {
                albedo,
                reflectivity,
            } => {
                let this = self.shade_diffuse(scene, albedo, surface.position, surface.normal);
                let reflected = self.cast_ray(
                    scene,
                    self.reflect_ray(ray.direction, surface.position, surface.normal),
                    depth + 1,
                );
                LinearRgb::mix(&this, &reflected, reflectivity)
            }
            Material::Refractive {
                albedo,
                index,
                transparency,
            } => {
                let kr = fresnel(ray.direction, surface.normal, index);
                let refracted = if kr < 1.0 {
                    self.cast_ray(
                        scene,
                        self.transmission_ray(
                            ray.direction,
                            surface.position,
                            surface.normal,
                            index,
                        ),
                        depth + 1,
                    )
                } else {
                    LinearRgb::BLACK
                };
                let reflected = self.cast_ray(
                    scene,
                    self.reflect_ray(ray.direction, surface.position, surface.normal),
                    depth + 1,
                );

                LinearRgb::mix(&(albedo * refracted * transparency), &reflected, kr)
            }
        }
    }

    fn shade_diffuse<S: Scene>(
        &self,
        scene: &S,
        albedo: LinearRgb,
        surface_position: Vec3,
        surface_normal: Dir3,
    ) -> LinearRgb {
        let mut result = LinearRgb::BLACK;

        for light in scene.lights() {
            match *light {
                Light::Ambient { color, intensity } => {
                    result += albedo * color * intensity;
                }
                Light::Directional {
                    direction,
                    color,
                    intensity,
                } => {
                    let dir_to_light = -direction;
                    let shadow_ray =
                        self.shadow_ray(surface_position, surface_normal, dir_to_light);
                    let light_intensity = match scene.cast_ray(shadow_ray, f32::INFINITY) {
                        Some(_) => continue,
                        None => intensity,
                    };
                    let light_power = surface_normal.dot(*dir_to_light).max(0.0) * light_intensity;

                    result += albedo * color * light_power / PI;
                }
                Light::Point {
                    position,
                    color,
                    intensity,
                } => {
                    let dir_to_light = Dir3::new(position - surface_position).unwrap();
                    let shadow_ray =
                        self.shadow_ray(surface_position, surface_normal, dir_to_light);
                    let distance_squared = Vec3::distance_squared(position, surface_position);
                    let light_intensity = match scene.cast_ray(shadow_ray, distance_squared.sqrt())
                    {
                        Some(_) => continue,
                        None => intensity / (4.0 * PI * distance_squared),
                    };
                    let light_power = surface_normal.dot(*dir_to_light).max(0.0) * light_intensity;

                    result += albedo * color * light_power / PI;
                }
            }
        }

        result
    }

    fn shadow_ray(
        &self,
        surface_position: Vec3,
        surface_normal: Dir3,
        dir_to_light: Dir3,
    ) -> Ray3d {
        Ray3d {
            origin: surface_position + self.shadow_bias * (*surface_normal + *dir_to_light),
            direction: dir_to_light,
        }
    }

    fn reflect_ray(&self, direction: Dir3, hit: Vec3, normal: Dir3) -> Ray3d {
        let direction = Dir3::new(*direction - (2.0 * direction.dot(*normal) * normal)).unwrap();
        Ray3d {
            origin: hit + self.shadow_bias * (*normal + *direction),
            direction,
        }
    }

    fn transmission_ray(&self, direction: Dir3, hit: Vec3, normal: Dir3, index: f32) -> Ray3d {
        let mut n = normal;
        let mut eta_t = index;
        let mut eta_i = 1.0;
        let mut i_dot_n = direction.dot(*normal);

        if i_dot_n < 0.0 {
            // Outside the surface
            i_dot_n = -i_dot_n;
        } else {
            // Inside the surface: invert normal and swap indices
            n = -normal;
            eta_t = 1.0;
            eta_i = index;
        }

        let eta = eta_i / eta_t;
        let k = 1.0 - (eta * eta) * (1.0 - i_dot_n * i_dot_n);

        let direction = Dir3::new((*direction + i_dot_n * n) * eta - n * k.sqrt()).unwrap();
        Ray3d {
            origin: hit + self.shadow_bias * (-*n + *direction),
            direction,
        }
    }
}

fn fresnel(direction: Dir3, normal: Dir3, index: f32) -> f32 {
    let dir_dot_n = direction.dot(*normal);
    let mut eta_i = 1.0;
    let mut eta_t = index;
    if dir_dot_n > 0.0 {
        eta_i = eta_t;
        eta_t = 1.0;
    }

    let sin_t = eta_i / eta_t * (1.0 - dir_dot_n * dir_dot_n).max(0.0).sqrt();
    if sin_t > 1.0 {
        1.0
    } else {
        let cos_t = (1.0 - sin_t * sin_t).max(0.0).sqrt();
        let cos_i = cos_t.abs();
        let r_s = ((eta_t * cos_i) - (eta_i * cos_t)) / ((eta_t * cos_i) + (eta_i * cos_t));
        let r_p = ((eta_i * cos_i) - (eta_t * cos_t)) / ((eta_i * cos_i) + (eta_t * cos_t));
        (r_s * r_s + r_p * r_p) / 2.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct LinearRgb {
    /// The red channel. [0.0, 1.0]
    pub red: f32,
    /// The green channel. [0.0, 1.0]
    pub green: f32,
    /// The blue channel. [0.0, 1.0]
    pub blue: f32,
}

impl LinearRgb {
    pub const BLACK: Self = Self {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };

    pub const WHITE: Self = Self {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };

    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }
}

impl From<LinearRgba> for LinearRgb {
    fn from(value: LinearRgba) -> Self {
        Self {
            red: value.red,
            green: value.green,
            blue: value.blue,
        }
    }
}

impl From<LinearRgb> for LinearRgba {
    fn from(value: LinearRgb) -> Self {
        Self {
            red: value.red,
            green: value.green,
            blue: value.blue,
            alpha: 1.0,
        }
    }
}

impl From<Color> for LinearRgb {
    fn from(value: Color) -> Self {
        Self::from(LinearRgba::from(value))
    }
}

impl From<LinearRgb> for Color {
    fn from(value: LinearRgb) -> Self {
        Color::LinearRgba(LinearRgba::from(value))
    }
}

impl Mix for LinearRgb {
    #[inline]
    fn mix(&self, other: &Self, factor: f32) -> Self {
        let n_factor = 1.0 - factor;
        Self {
            red: self.red * n_factor + other.red * factor,
            green: self.green * n_factor + other.green * factor,
            blue: self.blue * n_factor + other.blue * factor,
        }
    }
}

impl Add<Self> for LinearRgb {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
        }
    }
}

impl AddAssign<Self> for LinearRgb {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Mul<Self> for LinearRgb {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            red: self.red * rhs.red,
            green: self.green * rhs.green,
            blue: self.blue * rhs.blue,
        }
    }
}

impl Mul<f32> for LinearRgb {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}

impl Mul<LinearRgb> for f32 {
    type Output = LinearRgb;

    fn mul(self, rhs: LinearRgb) -> LinearRgb {
        LinearRgb {
            red: self * rhs.red,
            green: self * rhs.green,
            blue: self * rhs.blue,
        }
    }
}

impl Div<f32> for LinearRgb {
    type Output = LinearRgb;

    fn div(self, rhs: f32) -> LinearRgb {
        LinearRgb {
            red: self.red / rhs,
            green: self.green / rhs,
            blue: self.blue / rhs,
        }
    }
}
