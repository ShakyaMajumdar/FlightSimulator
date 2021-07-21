use macroquad::prelude::*;
use tobj;

struct Plane {
    mass: f32,
    position: Vec3,
    velocity: Vec3,
    mesh: Mesh,
}

impl Plane {
    fn new(mass: f32, position: Vec3, mesh: Mesh) -> Self {
        Self {
            mass,
            position,
            velocity: vec3(0., 0., 0.),
            mesh,
        }
    }
    fn draw(&self) {
        draw_mesh(&self.mesh);
    }
    fn update(&mut self, _dt: f32, _thrust: &Vec3, _wind: &Vec3) {}
}

#[macroquad::main("Flight Simulator")]
async fn main() {
    let mut camera = Camera3D {
        position: vec3(-20., 15., 0.),
        up: vec3(0., 1., 0.),
        target: vec3(0., 0., 0.),
        ..Default::default()
    };

    let (models, _materials) = tobj::load_obj(
        "plane.obj",
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
    )
    .expect("Failed to load");
    let texture = Some(Texture2D::from_file_with_format(
        include_bytes!("../texture.png"),
        None,
    ));

    let mesh = &models[0].mesh;
    println!("{}", mesh.positions.len());

    let mut vertices = Vec::new();
    for (vp, uv) in (0..mesh.positions.len() / 3).zip(0..mesh.texcoords.len() / 2) {
        vertices.push(macroquad::models::Vertex {
            position: vec3(
                mesh.positions[3 * vp],
                mesh.positions[3 * vp + 1],
                mesh.positions[3 * vp + 2],
            ),
            uv: vec2(mesh.texcoords[2 * uv], mesh.texcoords[2 * uv + 1]),
            color: Color::from_rgba(163, 190, 140, 255),
        })
    }

    let mut plane: Plane = Plane::new(
        100.,
        vec3(0., 0., 0.),
        Mesh {
            vertices,
            indices: mesh.indices.iter().map(|x| *x as u16).collect::<Vec<u16>>(),
            texture,
        },
    );
    loop {
        let dt = get_frame_time();

        clear_background(LIGHTGRAY);
        draw_grid(50, 1.0, RED, GREEN);

        let camera_up: Vec3 = (camera.up).normalize();
        let camera_forward: Vec3 = (camera.target - camera.position).normalize();
        let camera_right: Vec3 = (camera_forward.cross(camera_up)).normalize();

        if is_key_down(KeyCode::Escape) {
            break;
        }
        if is_key_down(KeyCode::W) {
            camera.position += camera_forward * 2.;
        }
        if is_key_down(KeyCode::S) {
            camera.position += camera_forward * -2.;
        }
        if is_key_down(KeyCode::D) {
            camera.position += camera_right * 2.;
        }
        if is_key_down(KeyCode::A) {
            camera.position += camera_right * -2.;
        }
        if is_key_down(KeyCode::Up) {
            camera.position += camera_up * 2.;
        }
        if is_key_down(KeyCode::Down) {
            camera.position += camera_up * -2.;
        }

        // camera.position += vec3(1., 1., 1.);
        set_camera(&camera);

        let thrust = vec3(0., 0., 0.);
        let wind = vec3(0., 0., 0.);
        plane.update(dt, &thrust, &wind);
        plane.draw();

        next_frame().await
    }
}
