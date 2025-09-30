use crate::{AppState, screens::ScreenSetup, util::exp_lerp};
use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit},
    prelude::*,
};
use bevy_spawn_observer::SpawnObserver;
use std::f32::consts::PI;

const LAG_WEIGHT: f32 = 0.75;

const DISTANCE_MIN: f32 = 0.1;
const DISTANCE_MAX: f32 = 50.0;

pub fn plugin(app: &mut App) {
    // Setup and cleanup
    app.add_systems(OnEnter(AppState::Game), setup.after(ScreenSetup));
    app.add_systems(OnExit(AppState::Game), cleanup);

    // Update
    app.add_systems(
        Update,
        (drag, update).chain().run_if(in_state(AppState::Game)),
    );
}

#[derive(Debug, Component)]
pub struct CameraController {
    orbit: Orbit,
    prev_look: Option<LookTransform>,
    is_dragging: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            orbit: Orbit::DEFAULT,
            prev_look: None,
            is_dragging: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Orbit {
    target: Transform,
    distance: f32,
    yaw: f32,
    pitch: f32,
}

impl Orbit {
    const DEFAULT: Self = Self {
        target: Transform::from_xyz(7.5, 0.0, 7.5),
        distance: 32.0,
        yaw: PI / 4.0,
        pitch: PI / 6.0,
    };
}

fn drag(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_controller: Single<&mut CameraController>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    let mouse_motion = mouse_motion_events
        .read()
        .fold(Vec2::ZERO, |acc, event| acc + event.delta);
    if !camera_controller.is_dragging {
        return;
    }

    let is_movement_allowed = true;
    let orbit = &mut camera_controller.orbit;
    if keyboard.pressed(KeyCode::ShiftLeft) {
        if is_movement_allowed {
            let mut delta = Quat::from_rotation_y(orbit.yaw)
                * Vec3::new(-mouse_motion.x * 0.01, 0.0, -mouse_motion.y * 0.01);
            delta.y = 0.0; // Ensure y is always 0
            orbit.target.translation += delta * orbit.distance / 50.0;
        }
    } else if keyboard.pressed(KeyCode::ControlLeft) {
        orbit.yaw = (orbit.yaw - mouse_motion.x * 0.002) % (2.0 * PI);
        orbit.pitch = f32::clamp(
            orbit.pitch + mouse_motion.y * 0.002,
            -(PI / 2.0 - 0.01),
            PI / 2.0 - 0.01,
        );
    }
}

fn update(mut cameras_query: Query<(&mut CameraController, &mut Transform)>, time: Res<Time>) {
    for (mut controller, mut transform) in &mut cameras_query {
        // Lag weight
        let s = exp_lerp(LAG_WEIGHT, time.delta_secs());

        // Calculate look transform
        let look = LookTransform::from_orbit(controller.orbit);
        let lerp_look = LookTransform::lerp(controller.prev_look.unwrap_or(look), look, s);
        controller.prev_look = Some(lerp_look);

        // Update transform
        *transform = lerp_look.into_transform();
    }
}

fn setup(mut commands: Commands, camera_controller: Single<Entity, With<CameraController>>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        GlobalZIndex(-10),
        UiTargetCamera(*camera_controller),
        StateScoped(AppState::Game),
        Children::spawn((
            SpawnObserver::new(
                |trigger: Trigger<Pointer<DragStart>>,
                 mut camera_controller: Single<&mut CameraController>| {
                    if trigger.button == PointerButton::Secondary {
                        camera_controller.is_dragging = true;
                    }
                },
            ),
            SpawnObserver::new(
                |trigger: Trigger<Pointer<DragEnd>>,
                 mut camera_controller: Single<&mut CameraController>| {
                    if trigger.button == PointerButton::Secondary {
                        camera_controller.is_dragging = false;
                    }
                },
            ),
        )),
    ));
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        GlobalZIndex(10),
        UiTargetCamera(*camera_controller),
        StateScoped(AppState::Game),
        Pickable {
            should_block_lower: false,
            is_hoverable: true,
        },
        Children::spawn(SpawnObserver::new(
            |trigger: Trigger<Pointer<Scroll>>,
             mut camera_controller: Single<&mut CameraController>| {
                let scroll = match trigger.unit {
                    MouseScrollUnit::Line => trigger.y / 5.0,
                    MouseScrollUnit::Pixel => trigger.y / 125.0 / 5.0,
                };
                camera_controller.orbit.distance = f32::clamp(
                    camera_controller.orbit.distance * (1.0 - scroll),
                    DISTANCE_MIN,
                    DISTANCE_MAX,
                );
            },
        )),
    ));
}

fn cleanup(mut _commands: Commands) {}

#[derive(Debug, Clone, Copy)]
struct LookTransform {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
}

impl LookTransform {
    fn from_orbit(orbit: Orbit) -> Self {
        Self {
            eye: orbit.target.translation
                + orbit.target.rotation
                    * Quat::from_euler(EulerRot::YXZ, orbit.yaw, -orbit.pitch, 0.0)
                    * (orbit.distance * Vec3::Z),
            target: orbit.target.translation,
            up: Vec3::Y,
        }
    }

    fn lerp(self, rhs: Self, s: f32) -> Self {
        Self {
            eye: Vec3::lerp(self.eye, rhs.eye, s),
            target: Vec3::lerp(self.target, rhs.target, s),
            up: Vec3::lerp(self.up, rhs.up, s),
        }
    }

    fn into_transform(self) -> Transform {
        Transform::from_translation(self.eye).looking_at(self.target, self.up)
    }
}
