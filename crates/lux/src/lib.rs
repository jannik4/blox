use bevy_color::prelude::*;
use bevy_math::prelude::*;

pub trait Scene {
    fn cast_ray(&self, ray: Ray3d) -> Option<RayHit>;
}

#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    pub material: Material,
    pub position: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum Material {
    Diffuse { albedo: LinearRgba },
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub translation: Vec3,
    pub direction: Dir3,
    pub up: Dir3,
    pub fov: f32,
    pub background: Color,
}

#[derive(Debug)]
pub struct Renderer {
    camera: Camera,
    dimensions: UVec2,

    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    top_left_pixel: Vec3,
}

impl Renderer {
    pub fn init(camera: Camera, dimensions: UVec2) -> Self {
        let focal_length = 1.0;
        let h = f32::tan(camera.fov / 2.0);
        let viewport_height = 2.0 * h * focal_length;
        let viewport_width = viewport_height * (dimensions.x as f32 / dimensions.y as f32);

        let w = -*camera.direction;
        let u = camera.up.cross(w);
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
        }
    }

    // TODO: Do in parallel on non-wasm
    pub fn render<S: Scene>(&self, scene: &S) -> Vec<Color> {
        let mut pixels = Vec::with_capacity((self.dimensions.x * self.dimensions.y) as usize);
        for y in 0..self.dimensions.y {
            for x in 0..self.dimensions.x {
                let pixel = self.render_pixel(scene, UVec2::new(x, y));
                pixels.push(pixel);
            }
        }
        pixels
    }

    pub fn render_pixel<S: Scene>(&self, scene: &S, pixel: UVec2) -> Color {
        let pixel = self.top_left_pixel
            + (pixel.x as f32) * self.pixel_delta_u
            + (pixel.y as f32) * self.pixel_delta_v;
        let ray = Ray3d {
            origin: self.camera.translation,
            direction: Dir3::new(pixel - self.camera.translation).unwrap(),
        };

        let hit = scene.cast_ray(ray);

        match hit {
            Some(hit) => match hit.material {
                Material::Diffuse { albedo } => albedo.into(),
            },
            None => self.camera.background,
        }
    }
}
