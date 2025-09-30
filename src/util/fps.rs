use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use std::fmt::Write;

pub fn plugin(app: &mut App) {
    if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    }

    app.add_systems(Startup, setup);
    app.add_systems(Update, update);
}

#[derive(Resource)]
struct FpsVisible(bool);

impl FpsVisible {
    fn display(&self) -> Display {
        match self.0 {
            true => Display::DEFAULT,
            false => Display::None,
        }
    }
}

impl Default for FpsVisible {
    fn default() -> Self {
        Self(cfg!(feature = "dev"))
    }
}

#[derive(Component)]
struct FpsText(Timer);

fn setup(mut commands: Commands) {
    let visible = FpsVisible::default();

    commands.spawn((
        FpsText(Timer::from_seconds(0.2, TimerMode::Repeating)),
        Text::default(),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextLayout {
            justify: JustifyText::Right,
            ..default()
        },
        Pickable::IGNORE,
        GlobalZIndex(1000),
        Node {
            display: visible.display(),
            position_type: PositionType::Absolute,
            bottom: Val::Px(4.0),
            right: Val::Px(4.0),
            ..default()
        },
    ));
    commands.insert_resource(visible);
}

fn update(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<FpsVisible>,
    mut text: Single<(&mut Node, &mut Text, &mut FpsText)>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        visible.0 = !visible.0;
        text.0.display = visible.display();
    }

    if !text.2.0.tick(time.delta()).just_finished() {
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .unwrap()
        .smoothed()
        .unwrap_or(0.0)
        .round() as i32;

    text.1.clear();
    write!(text.1, "{fps}").unwrap();
}
