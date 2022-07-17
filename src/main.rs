use std::f32::consts::TAU;

use bevy::{input::mouse::MouseMotion, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(AmbientLight {
            brightness: 0.5,
            color: Color::WHITE,
        })
        .add_startup_system(setup)
        .add_system(input_handling)
        .add_system(update_canvas_size)
        .add_system(normalize_gizmo_rot)
        .run();
}

fn update_canvas_size(mut windows: ResMut<Windows>) {
    let window_updated = windows.is_changed();
    #[cfg(not(target_arch = "wasm32"))]
    let update_window = || {};
    #[cfg(target_arch = "wasm32")]
    let mut update_window = || {
        let browser_window = web_sys::window()?;
        let window_width = browser_window.inner_width().ok()?.as_f64()?;
        let window_height = browser_window.inner_height().ok()?.as_f64()?;
        let window = windows.get_primary_mut()?;
        window.set_resolution(window_width as f32, window_height as f32);
        Some(())
    };
    if window_updated {
        update_window();
    }
}

#[derive(Component, PartialEq, Eq)]
enum Gizmo {
    Left,
    Right,
}

fn normalize_gizmo_rot(time: Res<Time>, mut gizmos: Query<&mut Transform, With<Gizmo>>) {
    let dt = time.delta_seconds_f64();
    let current = time.seconds_since_startup();
    if current % 1.0 < dt {
        for mut trans in gizmos.iter_mut() {
            trans.rotation = trans.rotation.normalize();
        }
    }
}

const ROT_SPEED: f32 = 0.01;
fn input_handling(
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut gizmos: Query<(&mut Transform, &Gizmo)>,
    windows: Res<Windows>,
) {
    use Gizmo::Right;

    if !mouse_buttons.pressed(MouseButton::Left) {
        return;
    }

    let window = if let Some(w) = windows.get_primary() {
        w
    } else {
        return;
    };
    for motion in mouse_motion.iter() {
        let window_width = window.physical_width();
        let cursor_pos = if let Some(w) = window.physical_cursor_position() {
            w
        } else {
            return;
        };
        let is_left = cursor_pos.x < window_width as f64 / 2.0;
        let rot = motion.delta * ROT_SPEED;
        for (mut gizmo_transform, gizmo) in gizmos.iter_mut() {
            if is_left ^ (*gizmo == Right) {
                let x_axis = gizmo_transform.rotation.inverse() * Vec3::X;
                let y_axis = gizmo_transform.rotation.inverse() * Vec3::Y;
                gizmo_transform.rotation *=
                    Quat::from_axis_angle(x_axis, rot.y) * Quat::from_axis_angle(y_axis, rot.x);
            }
        }
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mk_material = |base_color| StandardMaterial {
        base_color,
        perceptual_roughness: 0.9,
        reflectance: 0.1,
        ..default()
    };
    let handle = meshes.add(shape::Box::new(5.5, 0.5, 0.5).into());
    let sphere = meshes.add(shape::Icosphere::default().into());
    let green = materials.add(mk_material(Color::rgb_u8(0x66, 0xcc, 0x33)));
    let red = materials.add(mk_material(Color::rgb_u8(0xff, 0x33, 0x00)));
    let blue = materials.add(mk_material(Color::rgb_u8(0x00, 0x66, 0xcc)));
    let grey = materials.add(mk_material(Color::rgb_u8(0x99, 0x99, 0x99)));

    let mk_pbr = |mesh: &Handle<_>, mat: &Handle<_>, transform| PbrBundle {
        mesh: mesh.clone(),
        material: mat.clone(),
        transform,
        ..default()
    };
    let mk_ball = |material: &Handle<_>, translation| {
        mk_pbr(&sphere, material, Transform::from_translation(translation))
    };
    let mk_handle = |translation, rotation| {
        let transform = Transform {
            translation,
            rotation,
            ..default()
        };
        mk_pbr(&handle, &grey, transform)
    };
    let gizmo = |parent: &mut ChildBuilder, left_handed| {
        parent.spawn_bundle(mk_handle(Vec3::X * 2.5, Quat::default()));
        parent.spawn_bundle(mk_ball(&red, Vec3::X * 5.0));

        parent.spawn_bundle(mk_handle(Vec3::Y * 2.5, Quat::from_rotation_z(TAU / 4.0)));
        parent.spawn_bundle(mk_ball(&green, Vec3::Y * 5.0));

        let z_dir = if left_handed { -1.0 } else { 1.0 };
        let z_vec = Vec3::Z * 2.5 * z_dir;
        parent.spawn_bundle(mk_handle(z_vec, Quat::from_rotation_y(TAU / 4.0 * z_dir)));
        parent.spawn_bundle(mk_ball(&blue, z_vec * 2.0));
    };

    // GIZMOS
    let gizmo_pos = |x_pos| TransformBundle {
        local: Transform::from_xyz(x_pos, 0.0, 0.0),
        ..default()
    };

    commands
        .spawn_bundle(gizmo_pos(-6.5))
        .insert(Gizmo::Left)
        .with_children(|p| gizmo(p, true));
    commands
        .spawn_bundle(gizmo_pos(6.5))
        .insert(Gizmo::Right)
        .with_children(|p| gizmo(p, false));

    // lights
    let light = |z_pos| PointLightBundle {
        point_light: PointLight {
            intensity: 15000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 8.0, z_pos),
        ..default()
    };
    commands.spawn_bundle(light(10.0));
    commands.spawn_bundle(light(-10.0));

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 5.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
