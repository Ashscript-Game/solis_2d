use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::mouse::MouseWheel,
    prelude::*,
};
use bevy_egui::*;
use lommix_light::prelude::*;

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "radiant cascade".into(),
                    ..default()
                }),
                ..default()
            }),
            bevy_egui::EguiPlugin,
            LightPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(Startup, (setup, spawn_info_box))
        .add_systems(
            Update,
            (
                update,
                scroll,
                move_camera,
                move_light,
                spawn_light,
                clear,
                config,
                diagnostics,
                monitor,
            ),
        )
        .run();
}

#[derive(Component)]
struct Spin(f32);

#[derive(Component)]
struct FollowMouse;

fn monitor(diagnostics: Res<DiagnosticsStore>) {
    let Some(_fps) = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .map(|fps| fps.value())
        .flatten()
    else {
        return;
    };
}

fn setup(mut cmd: Commands, server: Res<AssetServer>) {
    cmd.spawn(RadianceCameraBundle {
        camera_bundle: Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 5.0))
                .looking_at(Vec3::default(), Vec3::Y),
            camera: Camera {
                clear_color: Color::BLACK.into(),
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::AcesFitted,
            ..default()
        },
        ..default()
    });

    for x in -4..=8 {
        for y in -4..=8 {
            cmd.spawn((
                SpriteBundle {
                    texture: server.load("box.png"),
                    transform: Transform::from_xyz((x as f32) * 400., (y as f32) * 400., 1.),
                    ..default()
                },
                Emitter {
                    intensity: 1.,
                    color: Color::BLACK,
                    shape: SdfShape::Rect(Vec2::new(50., 25.)),
                },
                Spin(rand::random::<f32>()),
            ));
        }
    }
    cmd.spawn((SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::splat(2000.)),
            ..default()
        },
        texture: server.load("box.png"),
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        ..default()
    },));

    cmd.spawn((
        Emitter {
            intensity: 1.,
            color: Color::WHITE,
            shape: SdfShape::Circle(200.),
        },
        FollowMouse,
        SpriteBundle {
            texture: server.load("lamp.png"),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
    ));
}

fn config(mut gi_config: Query<(&mut RadianceConfig, &mut RadianceDebug)>, mut egui: EguiContexts) {
    let Ok((mut gi_config, mut gi_debug)) = gi_config.get_single_mut() else {
        return;
    };

    egui::Window::new("Gi Config")
        .anchor(egui::Align2::RIGHT_TOP, [0., 0.])
        .show(egui.ctx_mut(), |ui| {
            ui.label("probe stride");
            ui.add(egui::Slider::new(&mut gi_config.probe_base, (2)..=16));
            ui.label("cascade count");
            ui.add(egui::Slider::new(&mut gi_config.cascade_count, (2)..=8));
            ui.label("interval");
            ui.add(egui::Slider::new(&mut gi_config.interval, (0.1)..=10.));
            ui.label("scale");
            ui.add(egui::Slider::new(&mut gi_config.scale_factor, (0.25)..=10.));

            flag_checkbox(GiFlags::DEBUG_SDF, ui, &mut gi_debug, "SDF");
            flag_checkbox(GiFlags::DEBUG_VORONOI, ui, &mut gi_debug, "VORONOI");
            flag_checkbox(GiFlags::DEBUG_MERGE1, ui, &mut gi_debug, "MERGE0");
            flag_checkbox(GiFlags::DEBUG_MERGE0, ui, &mut gi_debug, "MERGE1");
        });
}

#[derive(Default)]
struct Timings {
    count: u32,
    acc_fps: f64,
    acc_frame: f64,
    avg_fps: f64,
    avg_frame: f64,
}

fn diagnostics(
    mut egui: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    mut timings: Local<Timings>,
) {
    let Some(fps_time) = diagnostics.get_measurement(&FrameTimeDiagnosticsPlugin::FPS) else {
        return;
    };

    let Some(frame_time) = diagnostics.get_measurement(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
    else {
        return;
    };

    timings.acc_fps += fps_time.value;
    timings.acc_frame += frame_time.value;
    timings.count += 1;

    if timings.count == 100 {
        timings.avg_fps = timings.acc_fps * 0.01;
        timings.avg_frame = timings.acc_frame * 0.01;
        timings.count = 0;
        timings.acc_fps = 0.;
        timings.acc_frame = 0.;
    }

    egui::Window::new("Diagnostics")
        .anchor(egui::Align2::RIGHT_BOTTOM, [0., 0.])
        .show(egui.ctx_mut(), |ui| {
            ui.label(format!("FPS: {:.3}", timings.avg_fps));
            ui.label(format!("FRAME: {:.2}ms", timings.avg_frame));
        });
}

fn flag_checkbox(bit: GiFlags, ui: &mut egui::Ui, flags: &mut GiFlags, label: &str) {
    let mut state = (*flags & bit) != GiFlags::DEFAULT;
    ui.checkbox(&mut state, label);
    if state {
        *flags |= bit;
    } else {
        *flags &= !bit;
    }
}

fn update(mut query: Query<(&mut Transform, &Spin)>, time: Res<Time>) {
    query.iter_mut().for_each(|(mut transform, spin)| {
        transform.rotation = Quat::from_rotation_z(time.elapsed_seconds() * spin.0);
    });
}

fn spawn_info_box(mut cmd: Commands) {
    let mut node = NodeBundle::default();
    node.style.width = Val::Percent(100.);
    node.style.height = Val::Percent(100.);
    node.style.align_items = AlignItems::End;
    node.style.justify_content = JustifyContent::Start;

    cmd.spawn(node).with_children(|cmd| {
        let mut node = NodeBundle::default();
        node.style.border = UiRect::all(Val::Px(4.));
        node.background_color = BackgroundColor(Color::BLACK);
        node.border_radius = BorderRadius::all(Val::Px(15.));
        node.style.padding = UiRect::all(Val::Px(10.));
        cmd.spawn(node).with_children(|cmd| {
            cmd.spawn(TextBundle::from_section(
                "[Arrowkeys]:Move [I]:Zoom-in [O]:Zoom-out [Wheel]:Inc-size [Wheel+shift]:Inc-intensity",
                TextStyle {
                    color: Color::WHITE,
                    font_size: 20.,
                    ..default()
                },
            ));
        });
    });
}

fn spawn_light(
    mut cmd: Commands,
    mut light: Query<(&mut Emitter, &Transform), With<FollowMouse>>,
    inputs: Res<ButtonInput<MouseButton>>,
    mut egui: EguiContexts,
    server: Res<AssetServer>,
) {
    let Ok((mut emitter, transform)) = light.get_single_mut() else {
        return;
    };

    if egui.ctx_mut().is_pointer_over_area() {
        return;
    }

    if inputs.just_pressed(MouseButton::Left) {
        cmd.spawn((
            emitter.clone(),
            SpriteBundle {
                texture: server.load("lamp.png"),
                transform: transform.clone(),
                ..default()
            },
        ));

        let color = Color::srgb(
            rand::random::<f32>(),
            rand::random::<f32>(),
            rand::random::<f32>(),
        );

        emitter.color = color;
    }

    if inputs.just_pressed(MouseButton::Right) {
        cmd.spawn((
            Emitter {
                shape: emitter.shape.clone(),
                color: Color::BLACK,
                intensity: 1.,
            },
            SpriteBundle {
                texture: server.load("lamp.png"),
                transform: transform.clone(),
                ..default()
            },
        ));
    }
}

fn clear(
    mut cmd: Commands,
    emitters: Query<Entity, (Without<FollowMouse>, Without<Spin>, With<Emitter>)>,
    inputs: Res<ButtonInput<KeyCode>>,
) {
    if inputs.just_pressed(KeyCode::KeyR) {
        emitters
            .iter()
            .for_each(|entity| cmd.entity(entity).despawn_recursive());
    }
}

fn scroll(
    mut emitters: Query<&mut Emitter, With<FollowMouse>>,
    mut events: EventReader<MouseWheel>,
    inputs: Res<ButtonInput<KeyCode>>,
    mut multi: Local<f32>,
) {
    let Some(event) = events.read().next() else {
        return;
    };

    let Ok(mut emitter) = emitters.get_single_mut() else {
        return;
    };

    let dir = event.y.signum();
    match inputs.pressed(KeyCode::ShiftLeft) {
        true => {
            emitter.intensity = (emitter.intensity + dir * 0.1).max(0.);
        }
        false => {
            *multi = (*multi + dir).max(0.);
            emitter.shape = SdfShape::Circle(*multi);
        }
    }
}

fn move_camera(mut camera: Query<&mut Transform, With<Camera>>, inputs: Res<ButtonInput<KeyCode>>) {
    let Ok(mut transform) = camera.get_single_mut() else {
        return;
    };

    transform.translation += Vec3::new(
        (inputs.pressed(KeyCode::ArrowRight) as i32 - inputs.pressed(KeyCode::ArrowLeft) as i32)
            as f32,
        (inputs.pressed(KeyCode::ArrowUp) as i32 - inputs.pressed(KeyCode::ArrowDown) as i32)
            as f32,
        0.,
    ) * 2.;

    if inputs.just_pressed(KeyCode::KeyO) {
        transform.scale += Vec3::splat(0.05);
    }
    if inputs.just_pressed(KeyCode::KeyI) {
        transform.scale -= Vec3::splat(0.05);
    }
}

fn move_light(
    window: Query<&Window>,
    camera: Query<Entity, With<Camera2d>>,
    light: Query<Entity, With<FollowMouse>>,
    mut transforms: Query<&mut Transform>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position().map(|pos| {
        let offset = transforms
            .get(camera.single())
            .unwrap()
            .translation
            .truncate();
        (pos - Vec2::new(window.width() / 2., window.height() / 2.)) * Vec2::new(1., -1.) + offset
    }) else {
        return;
    };

    let scale = transforms.get(camera.single()).unwrap().scale.x;
    let Some(mut light_transform) = light
        .get_single()
        .ok()
        .map(|entity| transforms.get_mut(entity).ok())
        .flatten()
    else {
        return;
    };

    light_transform.translation.x = cursor_pos.x * scale;
    light_transform.translation.y = cursor_pos.y * scale;
}
