use bevy::{prelude::*, input::mouse::MouseMotion, render::{options::WgpuOptions, render_resource::{WgpuFeatures, PrimitiveTopology}, mesh::Indices, view::VisibleEntities, primitives::Frustum}, pbr::wireframe::{WireframePlugin, WireframeConfig}, sprite::MaterialMesh2dBundle};
use image::{ImageBuffer, Luma, ImageError};

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
        //.add_system(spin_object)
        .run();
}

#[derive(Component)]
struct Object;
/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {
    // To draw the wireframe on all entities, set this to 'true'
    wireframe_config.global = false;

    // add entities to the world
    let texture_handle = asset_server.load("unwrap_helper.png");

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Opaque,
        unlit: false,
        ..Default::default()
    });

    // lights
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(5.0, 5.0, 5.0),
        ..Default::default()
    });
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.25,
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    }).insert(CameraController::default());

    //commands.spawn_bundle(OrthographicCameraBundle::new_3d())
    //.insert(CameraController::default());


    // terrain
    let terrain_mesh = load_terrain_bitmap("terrain.png", TerrainImageLoadOptions { max_image_height: 0.25, pixel_side_length: 0.1 }).unwrap();
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(terrain_mesh)),
        material: material_handle,
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
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    let size = options.pixel_side_length;
    let mut i: u32 = 0;

    for y in 0..heightmap.height() {
        for x in 0..heightmap.width() {
            let height = sample_vertex_height(y as i32, x as i32, heightmap);
            let height_down = sample_vertex_height((y as i32 + 1).min(heightmap.height() as i32), x as i32, heightmap);
            let height_right = sample_vertex_height(y as i32, (x as i32 + 1).min(heightmap.width() as i32), heightmap);

            // top face
            vertices.push([x as f32 * size, height * options.max_image_height, y as f32 * size]);
            vertices.push([(x + 1) as f32 * size, height * options.max_image_height, y as f32 * size]);
            vertices.push([x as f32 * size, height * options.max_image_height, (y + 1) as f32 * size]);
            vertices.push([(x + 1) as f32 * size, height * options.max_image_height, (y + 1) as f32 * size]);
            // right face
            vertices.push([(x + 1) as f32 * size, height * options.max_image_height, y as f32 * size]);
            vertices.push([(x + 1) as f32 * size, height * options.max_image_height, (y + 1) as f32 * size]);
            vertices.push([(x + 1) as f32 * size, height_right * options.max_image_height, y as f32 * size]);
            vertices.push([(x + 1) as f32 * size, height_right * options.max_image_height, (y + 1) as f32 * size]);
            // bottom face
            vertices.push([x as f32 * size, height * options.max_image_height, (y + 1) as f32 * size]);
            vertices.push([(x + 1) as f32 * size, height * options.max_image_height, (y + 1) as f32 * size]);
            vertices.push([x as f32 * size, height_down * options.max_image_height, (y + 1) as f32 * size]);
            vertices.push([(x + 1) as f32 * size, height_down * options.max_image_height, (y + 1) as f32 * size]);
            // uvs - top face
            uvs.push([0.0, 0.0]);
            uvs.push([1.0, 0.0]);
            uvs.push([0.0, 1.0]);
            uvs.push([1.0, 1.0]);
            // NOTE(matt): these might need flipped if the face is the wrong way, just like the normals need flipped
            // uvs - right face
            uvs.push([1.0, 0.0]);
            uvs.push([0.0, 0.0]);
            uvs.push([1.0, 1.0]);
            uvs.push([0.0, 1.0]);
            // uvs - bottom face
            uvs.push([0.0, 0.0]);
            uvs.push([1.0, 0.0]);
            uvs.push([0.0, 1.0]);
            uvs.push([1.0, 1.0]);
            // normals
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
            // flip normals if needed
            let d = if height > height_right { 1.0 } else { -1.0 };
            normals.push([d, 0.0, 0.0]);
            normals.push([d, 0.0, 0.0]);
            normals.push([d, 0.0, 0.0]);
            normals.push([d, 0.0, 0.0]);
            let d = if height > height_down { 1.0 } else { -1.0 };
            normals.push([0.0, 0.0, d]);
            normals.push([0.0, 0.0, d]);
            normals.push([0.0, 0.0, d]);
            normals.push([0.0, 0.0, d]);
            // top face
            indices.push(i);
            indices.push(i+3);
            indices.push(i+1);

            indices.push(i);
            indices.push(i+2);
            indices.push(i+3);

            // right face?
            indices.push(i+4);
            indices.push(i+5);
            indices.push(i+7);

            indices.push(i+4);
            indices.push(i+7);
            indices.push(i+6);
            //// bottom face?
            indices.push(i+8);
            indices.push(i+10);
            indices.push(i+11);

            indices.push(i+8);
            indices.push(i+11);
            indices.push(i+9);

            i += 12; // for our vertex stride
        }
    }

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

fn spin_object(
    time: Res<Time>,
    mut q: Query<&mut Transform, With<Object>>
){
    let dt = time.delta_seconds();

    let mut t = q.single_mut();
    t.rotate(Quat::from_rotation_y(0.25 * dt));
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