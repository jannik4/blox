use bevy_color::prelude::*;
use bevy_math::prelude::*;
use std::{f32::consts::PI, thread};

pub trait Scene {
    fn lights(&self) -> &[Light];
    fn cast_ray(&self, ray: Ray3d, max_distance: f32) -> Option<RayHit>;
}

#[derive(Debug, Clone, Copy)]
pub enum Light {
    Ambient {
        color: LinearRgba,
        intensity: f32,
    },
    Directional {
        direction: Dir3,
        color: LinearRgba,
        intensity: f32,
    },
    Point {
        position: Vec3,
        color: LinearRgba,
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
        albedo: LinearRgba,
    },
    Reflective {
        albedo: LinearRgba,
        reflectivity: f32,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub translation: Vec3,
    pub direction: Dir3,
    pub up: Dir3,
    pub fov: f32,
    pub background: LinearRgba,
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

    fn cast_ray<S: Scene>(&self, scene: &S, ray: Ray3d, depth: u32) -> LinearRgba {
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
                LinearRgba::mix(&this, &reflected, reflectivity)
            }
        }
    }

    fn shade_diffuse<S: Scene>(
        &self,
        scene: &S,
        albedo: LinearRgba,
        surface_position: Vec3,
        surface_normal: Dir3,
    ) -> LinearRgba {
        let mut result = LinearRgba::BLACK;

        for light in scene.lights() {
            match *light {
                Light::Ambient { color, intensity } => {
                    result += apply_light(albedo, color * intensity);
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

                    result += apply_light(albedo, color * light_power / PI);
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

                    result += apply_light(albedo, color * light_power / PI);
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
}

fn apply_light(color: LinearRgba, light: LinearRgba) -> LinearRgba {
    LinearRgba::new(
        color.red * light.red,
        color.green * light.green,
        color.blue * light.blue,
        color.alpha,
    )
}
