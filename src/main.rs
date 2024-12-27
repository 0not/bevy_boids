use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;

use bevy_boids::voxel::VoxelHashMap;

const BOID_NUMBER: usize = 10000;
const BOID_RADIUS: f32 = 10.0;
const BOID_SECTION_DEG: f32 = 10.0;
const BOID_MAX_SPEED: f32 = 600.0;
const BOID_MIN_SPEED: f32 = 50.0;
const BOID_SEPARATION_FACTOR: f32 = 0.05;
const BOID_ALIGNMENT_FACTOR: f32 = 0.05;
const BOID_COHESION_FACTOR: f32 = 0.005;
const BOID_TURN_FACTOR: f32 = 5.;
const BOID_SEPARATION_RADIUS: f32 = 10.;
const BOID_ALIGNMENT_RADIUS: f32 = 40.;

const DEBUG: bool = false;

#[derive(Component, PartialEq)]
struct Boid {
    avoid_radius: f32,
    align_radius: f32,
    separation_accumulator: Vec3,
    alignment_accumulator: Vec3,
    position_accumulator: Vec3,
    n_neighbors: usize,
}

#[derive(Component)]
struct Velocity(Vec3);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
        .add_systems(Startup, (setup, spawn_boids).chain())
        // .add_systems(Update, (avoid_boundary, move_boids).chain())
        .add_systems(
            Update,
            (boids_behavior, avoid_boundary, move_boids).chain(), // color_boids
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.insert_resource(VoxelHashMap {
        map: Default::default(),
        cell_size: 2. * BOID_ALIGNMENT_RADIUS / 3., // 3 cells should equal alignment diameter
                                                    // cell_size: BOID_SEPARATION_RADIUS / 2.,
    });

    // Render voxel cell boundaries
    if DEBUG {
        // commands.spawn();
    }
}

fn spawn_boids(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut voxels: ResMut<VoxelHashMap>,
) {
    let shape = meshes.add(CircularSector::new(
        BOID_RADIUS,
        f32::to_radians(BOID_SECTION_DEG),
    ));
    let inner = meshes.add(Annulus::new(
        BOID_SEPARATION_RADIUS - 1.,
        BOID_SEPARATION_RADIUS,
    ));
    let outer = meshes.add(Annulus::new(
        BOID_ALIGNMENT_RADIUS - 1.,
        BOID_ALIGNMENT_RADIUS,
    ));

    for _ in 0..BOID_NUMBER {
        let x = rand::random::<f32>() * 800. - 400.;
        let y = rand::random::<f32>() * 600. - 300.;
        let translation = Vec3::new(x, y, 0.);
        // let vx = rand::random::<f32>() * BOID_MAX_SPEED - BOID_MAX_SPEED / 2.;
        // let vy = rand::random::<f32>() * BOID_MAX_SPEED - BOID_MAX_SPEED / 2.;
        // let v = translation.cross(Vec3::Z).normalize() * BOID_MAX_SPEED / 6.;
        let v = Vec3::ZERO;

        let entity = commands
            .spawn((
                Boid {
                    avoid_radius: BOID_SEPARATION_RADIUS,
                    align_radius: BOID_ALIGNMENT_RADIUS,
                    separation_accumulator: Vec3::ZERO,
                    alignment_accumulator: Vec3::ZERO,
                    position_accumulator: Vec3::ZERO,
                    n_neighbors: 0,
                },
                Mesh2d(shape.clone()),
                MeshMaterial2d(materials.add(Color::WHITE)),
                Transform::from_translation(translation),
                Velocity(v),
            ))
            .with_children(|parent| {
                parent.spawn_empty().insert_if(
                    (
                        Mesh2d(inner.clone()),
                        MeshMaterial2d(materials.add(Color::linear_rgba(1., 0., 0., 0.1))),
                        Transform::from_xyz(0., 0., 0.),
                    ),
                    || DEBUG,
                );
                parent.spawn_empty().insert_if(
                    (
                        Mesh2d(outer.clone()),
                        MeshMaterial2d(materials.add(Color::linear_rgba(0., 1., 0., 0.1))),
                        Transform::from_xyz(0., 0., 0.),
                    ),
                    || DEBUG,
                );
            })
            .id();

        // Add to voxel hash map
        voxels.insert(translation.xy(), entity);
    }
    println!("Number of voxels: {}", voxels.map.len());
}

fn boids_behavior_fast(
    mut q_boids: Query<(&mut Boid, &Transform, &Velocity)>,
    voxels: Res<VoxelHashMap>,
) {
    let mut velocity: HashMap<(i64, i64), Vec2> = HashMap::new();
    let mut position: HashMap<(i64, i64), Vec2> = HashMap::new();
    // let mut n_neighbors: HashMap<(i64, i64), usize> = HashMap::new();

    for (pos, entities) in voxels.map.iter() {
        let (x, y) = pos;
        let mut n = 0;
        for entity in entities {
            if let Ok((_boid, transform, vel)) = q_boids.get(*entity) {
                n += 1;
                *velocity.entry((*x, *y)).or_insert(vel.0.xy()) += vel.0.xy();
                *position
                    .entry((*x, *y))
                    .or_insert(transform.translation.xy()) += transform.translation.xy();
            }
        }
        *velocity.entry((*x, *y)).or_insert(Vec2::ZERO) /= n as f32;
        *position.entry((*x, *y)).or_insert(Vec2::ZERO) /= n as f32;
        // n_neighbors.insert((*x, *y), n);
    }

    for (mut boid, transform, _) in q_boids.iter_mut() {
        let mut separation_acc = Vec2::ZERO;
        let mut alignment_acc = Vec2::ZERO;
        let mut position_acc = Vec2::ZERO;

        let mut n_neighbors = 0;
        let neighbors =
            voxels.get_neighbor_keys_within(transform.translation.xy(), BOID_ALIGNMENT_RADIUS);

        for key in neighbors {
            let Some(other_position) = position.get(&key) else {
                continue;
            };
            let Some(other_velocity) = velocity.get(&key) else {
                continue;
            };

            let distance = transform.translation.xy().distance_squared(*other_position);
            if distance < boid.avoid_radius * boid.avoid_radius {
                separation_acc += transform.translation.xy() - other_position;
            } else if distance < boid.align_radius * boid.align_radius {
                alignment_acc += other_velocity;
                position_acc += other_position;
                n_neighbors += 1;
            }
        }

        boid.separation_accumulator = separation_acc.extend(0.);
        boid.alignment_accumulator = alignment_acc.extend(0.);
        boid.position_accumulator = position_acc.extend(0.);
        boid.n_neighbors = n_neighbors;
    }
}

fn boids_behavior(
    mut q_boids: Query<(&mut Boid, &Transform)>,
    q_boids_other: Query<(&Transform, &Velocity), With<Boid>>,
    voxels: Res<VoxelHashMap>,
) {
    for (mut boid, transform) in q_boids.iter_mut() {
        let mut separation = Vec3::ZERO;
        let mut alignment = Vec3::ZERO;
        let mut position = Vec3::ZERO;

        let mut n_neighbors = 0;

        let neighbors = voxels.get_neighbor_entities(transform.translation.xy());

        // for (other_transform, other_velocity) in q_boids_other.iter() {
        for other_entity in neighbors {
            let (other_transform, other_velocity) = q_boids_other.get(other_entity).unwrap();
            // if boid == other_boid {
            //     continue;
            // }
            // Separation
            let distance = transform
                .translation
                .distance_squared(other_transform.translation);
            if distance < boid.avoid_radius * boid.avoid_radius {
                separation += transform.translation - other_transform.translation;
            }
            // Alignment
            else if distance < boid.align_radius * boid.align_radius {
                alignment += other_velocity.0;
                position += other_transform.translation;
                n_neighbors += 1;
            }
        }

        boid.separation_accumulator = separation;
        boid.alignment_accumulator = alignment;
        boid.position_accumulator = position;
        boid.n_neighbors = n_neighbors;

        // // Separation
        // velocity.0 += separation * BOID_SEPARATION_FACTOR;

        // if n_neighbors > 0 {
        //     alignment /= n_neighbors as f32;
        //     average /= n_neighbors as f32;

        //     // Alignment
        //     let vel = velocity.0;
        //     velocity.0 += (alignment - vel) * BOID_ALIGNMENT_FACTOR;

        //     // Cohesion
        //     velocity.0 += (average - transform.translation) * BOID_COHESION_FACTOR;
        // }
    }
}

fn move_boids(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Boid, &mut Transform, &mut Velocity)>,
    mut voxels: ResMut<VoxelHashMap>,
) {
    for (entity, mut boid, mut transform, mut velocity) in query.iter_mut() {
        // Separation
        velocity.0 += boid.separation_accumulator * BOID_SEPARATION_FACTOR;

        let n_neighbors = boid.n_neighbors;
        if n_neighbors > 0 {
            boid.alignment_accumulator /= n_neighbors as f32;
            boid.position_accumulator /= n_neighbors as f32;

            // Alignment
            let vel = velocity.0;
            velocity.0 += (boid.alignment_accumulator - vel) * BOID_ALIGNMENT_FACTOR;

            // Cohesion
            velocity.0 +=
                (boid.position_accumulator - transform.translation) * BOID_COHESION_FACTOR;
        }

        // Reset values
        boid.separation_accumulator = Vec3::ZERO;
        boid.alignment_accumulator = Vec3::ZERO;
        boid.position_accumulator = Vec3::ZERO;
        boid.n_neighbors = 0;

        // Cap the velocity
        if velocity.0.length() > BOID_MAX_SPEED {
            velocity.0 = velocity.0.normalize() * BOID_MAX_SPEED;
        }

        if velocity.0.length() < BOID_MIN_SPEED {
            velocity.0 = velocity.0.normalize() * BOID_MIN_SPEED;
        }

        let old_translation = transform.translation.xy();
        transform.translation += velocity.0 * time.delta_secs();
        voxels.update_entity(old_translation, transform.translation.xy(), entity);

        // Rotate to face the direction of the velocity vector
        let angle = ops::atan2(velocity.0.y, velocity.0.x);
        transform.rotation = Quat::from_rotation_z(angle + std::f32::consts::FRAC_PI_2);
    }
}

fn color_boids(
    mut query: Query<(&Velocity, &MeshMaterial2d<ColorMaterial>), With<Boid>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // let u = (Vec3::new(230., 132., 96.) / 255.).normalize();
    let u = (Vec3::new(96., 230., 125.) / 255.).normalize();
    let v = (Vec3::new(96., 181., 230.) / 255.).normalize();
    let w = (Vec3::new(1., 0., 0.)).normalize();
    // let w = (Vec3::new(230., 132., 96.) / 255.).normalize();

    for (velocity, material) in query.iter_mut() {
        let v_norm = (velocity.0.normalize() + 1.) / 2.;
        // let hue = 360. * (velocity.0.length() - BOID_MIN_SPEED) / (BOID_MAX_SPEED - BOID_MIN_SPEED);
        // let saturation = (v_norm.x + 1.) / 2.;
        // let lightness = (v_norm.y + 1.) / 2.;
        // let color = Color::hsl(hue, saturation, lightness);
        let c = v_norm.x * u
            + v_norm.y * v
            + 0.3
                * (1. - (velocity.0.length() - BOID_MIN_SPEED) / (BOID_MAX_SPEED - BOID_MIN_SPEED))
                * w;
        // let color = Color::xyz(
        //     (v_norm.x + 1.) / 2.,
        //     (v_norm.y + 1.) / 2.,
        //     (velocity.0.length() - BOID_MIN_SPEED) / (BOID_MAX_SPEED - BOID_MIN_SPEED),
        // );
        let color = Color::linear_rgb(c.x, c.y, c.z);
        let color_mat = materials.get_mut(material).unwrap();
        color_mat.color = color;
    }
}

// fn rotate_boids(mut query: Query<(&Boid, &mut Transform, &Velocity)>) {
//     for (_, mut transform, velocity) in query.iter_mut() {
//         // Rotate to face the direction of the velocity vector
//         let angle = ops::atan2(velocity.0.y, velocity.0.x);
//         transform.rotation = Quat::from_rotation_z(angle + std::f32::consts::FRAC_PI_2);
//     }
// }

fn avoid_boundary(
    mut query: Query<(&mut Velocity, &Transform), With<Boid>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let (width, height) = match window_q.get_single() {
        Ok(window) => (
            window.width() / 2. - 5. * BOID_RADIUS,
            window.height() / 2. - 5. * BOID_RADIUS,
        ),
        _ => (400., 400.),
    };

    for (mut velocity, transform) in query.iter_mut() {
        let x = transform.translation.x;
        let y = transform.translation.y;

        if x > width {
            velocity.0.x -= BOID_TURN_FACTOR;
        }
        if x < -width {
            velocity.0.x += BOID_TURN_FACTOR;
        }
        if y > height {
            velocity.0.y -= BOID_TURN_FACTOR;
        }
        if y < -height {
            velocity.0.y += BOID_TURN_FACTOR;
        }
    }
}

#[allow(dead_code)]
fn periodic_boundary(
    mut query: Query<&mut Transform, With<Boid>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let (width, height) = match window_q.get_single() {
        Ok(window) => (
            window.width() / 2. + BOID_RADIUS,
            window.height() / 2. + BOID_RADIUS,
        ),
        _ => (400., 400.),
    };

    for mut transform in query.iter_mut() {
        if transform.translation.x > width {
            transform.translation.x = -width;
        }
        if transform.translation.x < -width {
            transform.translation.x = width;
        }
        if transform.translation.y > height {
            transform.translation.y = -height;
        }
        if transform.translation.y < -height {
            transform.translation.y = height;
        }
    }
}
