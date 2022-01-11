use bevy::{prelude::*, input::mouse::MouseMotion, render::{options::WgpuOptions, render_resource::{WgpuFeatures, PrimitiveTopology}, mesh::Indices}, pbr::wireframe::{WireframePlugin, WireframeConfig}};
use image::{ImageBuffer, Luma, ImageError};
//use terrain::load_terrain_bitmap;

//mod terrain;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .add_system(camera_controller)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // To draw the wireframe on all entities, set this to 'true'
    wireframe_config.global = true;

    // add entities to the world
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    // lights
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 15000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(20.0, 20.0, 20.0),
        ..Default::default()
    });
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    }).insert(CameraController::default());


    // terrain
    let terrain_mesh = load_terrain_bitmap("terrain.png", TerrainImageLoadOptions { max_image_height: 3.0, pixel_side_length: 0.1 }).unwrap();
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(terrain_mesh)),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        //transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    });
}

#[derive(Default)]
pub struct TerrainImageLoadOptions {
    pub max_image_height : f32,
    pub pixel_side_length : f32
}

fn sample_vertex_height(cy: i32, cx: i32, heightmap: &ImageBuffer<Luma<u16>, Vec::<u16>>) -> f32 {
    let mut cnt = 0;
    let mut height = 0.0;

    for dy in [-1, 0].iter() {
        for dx in [-1, 0].iter() {
            let sy = cy + dy;
            let sx = cx + dx;
            if    sy < 0 
               || sx < 0 
               || sy >= heightmap.height() as i32 
               || sx >= heightmap.width() as i32 {
                continue;
            } else {
                height += heightmap.get_pixel(
                    sx as u32, sy as u32).0[0] as f32 * 1.0f32 / std::u16::MAX as f32;
                cnt += 1;
            }
        }
    }

    height / cnt as f32
}

fn load_terrain_bitmap(filename: &str, options: TerrainImageLoadOptions) -> Result<Mesh, ImageError> {
    let terrain_bitmap = image::open(filename)?;

    let heightmap = terrain_bitmap.as_luma16().unwrap();

    let mut vertices = Vec::new();

    for y in 0..heightmap.height() {
        for x in 0..heightmap.width() {
            let height = sample_vertex_height(y as i32, x as i32, heightmap);
            vertices.push([x as f32, height * options.max_image_height, y as f32])
        }
    }

    let mut indices = Vec::new();

    for y in 0..heightmap.height()-1 {
        for x in 0..heightmap.width()-1 {
            // 2 triangles per cell
            indices.push(y * heightmap.width() + x);
            indices.push((y+1) * heightmap.width() + x+1);
            indices.push(y * heightmap.width() + x+1);

            indices.push(y * heightmap.width() + x);
            indices.push((y+1) * heightmap.width() + x);
            indices.push((y+1) * heightmap.width() + x+1);
        }
    }

    let normals = vec![[0.0, 1.0, 0.0]; vertices.len()];
    let uvs = vec![[0.0, 0.0]; vertices.len()];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    Ok(mesh)    
}

#[derive(Component)]
struct CameraController {
    pub enabled: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: 0.5,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::E,
            key_down: KeyCode::Q,
            key_run: KeyCode::LShift,
            walk_speed: 10.0,
            run_speed: 30.0,
            friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
        }
    }
}

fn camera_controller(
    time: Res<Time>,
    mut mouse_events: EventReader<MouseMotion>,
    key_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    let dt = time.delta_seconds();

    // Handle mouse input
    let mut mouse_delta = Vec2::ZERO;
    for mouse_event in mouse_events.iter() {
        mouse_delta += mouse_event.delta;
    }

    for (mut transform, mut options) in query.iter_mut() {
        if !options.enabled {
            continue;
        }

        // Handle key input
        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(options.key_forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(options.key_back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(options.key_right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(options.key_left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(options.key_up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(options.key_down) {
            axis_input.y -= 1.0;
        }

        // Apply movement update
        if axis_input != Vec3::ZERO {
            let max_speed = if key_input.pressed(options.key_run) {
                options.run_speed
            } else {
                options.walk_speed
            };
            options.velocity = axis_input.normalize() * max_speed;
        } else {
            let friction = options.friction.clamp(0.0, 1.0);
            options.velocity *= 1.0 - friction;
            if options.velocity.length_squared() < 1e-6 {
                options.velocity = Vec3::ZERO;
            }
        }
        let forward = transform.forward();
        let right = transform.right();
        transform.translation += options.velocity.x * dt * right
            + options.velocity.y * dt * Vec3::Y
            + options.velocity.z * dt * forward;

        if mouse_delta != Vec2::ZERO {
            // Apply look update
            let (pitch, yaw) = (
                (options.pitch - mouse_delta.y * 0.5 * options.sensitivity * dt).clamp(
                    -0.99 * std::f32::consts::FRAC_PI_2,
                    0.99 * std::f32::consts::FRAC_PI_2,
                ),
                options.yaw - mouse_delta.x * options.sensitivity * dt,
            );
            transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch);
            options.pitch = pitch;
            options.yaw = yaw;
        }
    }
}