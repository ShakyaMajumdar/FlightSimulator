use macroquad::prelude::*;

struct Plane {
    mass: f32,
    head: Vec3,
    center: Vec3,
    right_wing_tip: Vec3,
    velocity: Vec3,
    mesh: macroquad::models::Mesh,
    camera: Camera3D,
}

impl Plane {
    fn new(mass: f32, mesh: Mesh) -> Self {
        let mut plane = Self {
            mass,
            head: _get_head_from_vertices(&mesh.vertices),
            center: _get_center_from_vertices(&mesh.vertices),
            right_wing_tip: _get_right_wing_tip_from_vertices(&mesh.vertices),
            velocity: vec3(0., 0., 0.),
            mesh,
            camera: Camera3D {
                ..Default::default()
            },
        };
        plane.camera.position = plane.up() * 2. + plane.forward() * -3.;
        plane.camera.target = plane.forward() * 3.;
        plane.camera.up = plane.up();
        plane
    }
    fn draw(&self) {
        draw_mesh(&self.mesh);
    }
    fn translate_by(&mut self, move_vector: Vec3) {
        for vertex in &mut self.mesh.vertices {
            vertex.position += move_vector;
        }
        self.head += move_vector;
        self.right_wing_tip += move_vector;
        self.center += move_vector;
        self.camera.position += move_vector;
        self.camera.target += move_vector;
        self.camera.up = self.up();
    }

    fn rotate_by(&mut self, axis: &Vec3, angle: f32) {
        let rotation_matrix = glam::Mat3::from_axis_angle(*axis, angle);
        for vertex in &mut self.mesh.vertices {
            vertex.position = rotation_matrix.mul_vec3(vertex.position);
        }
        self.head = rotation_matrix.mul_vec3(self.head);
        self.right_wing_tip = rotation_matrix.mul_vec3(self.right_wing_tip);
        self.center = rotation_matrix.mul_vec3(self.center);
        self.camera.position = rotation_matrix.mul_vec3(self.camera.position);
        self.camera.target = rotation_matrix.mul_vec3(self.camera.target);
        self.camera.up = self.up()
    }

    fn update(&mut self, _dt: f32, _thrust: &Vec3, _wind: &Vec3) {}
    fn up(&self) -> Vec3 {
        (self.right_wing_tip - self.center)
            .cross(self.head - self.center)
            .normalize()
    }
    fn forward(&self) -> Vec3 {
        (self.head - self.center).normalize()
    }
    fn right(&self) -> Vec3 {
        (self.right_wing_tip - self.center).normalize()
    }
}

fn _get_vertices_from_mesh(mesh: &tobj::Mesh) -> Vec<macroquad::models::Vertex> {
    (0..mesh.positions.len() / 3)
        .zip(0..mesh.texcoords.len() / 2)
        .map(|(v_index, uv)| macroquad::models::Vertex {
            position: vec3(
                mesh.positions[3 * v_index],
                mesh.positions[3 * v_index + 1],
                mesh.positions[3 * v_index + 2],
            ),
            uv: vec2(mesh.texcoords[2 * uv], mesh.texcoords[2 * uv + 1]),
            color: Color::from_rgba(127, 127, 127, 255),
        })
        .collect::<Vec<macroquad::models::Vertex>>()
}

fn _get_head_from_vertices(vertices: &Vec<macroquad::models::Vertex>) -> macroquad::math::Vec3 {
    vertices
        .iter()
        .map(|vertex| vertex.position)
        .max_by(|this, other| this.x.partial_cmp(&other.x).unwrap())
        .unwrap()
}

fn _get_right_wing_tip_from_vertices(
    vertices: &Vec<macroquad::models::Vertex>,
) -> macroquad::math::Vec3 {
    vertices
        .iter()
        .map(|vertex| vertex.position)
        .max_by_key(|position| (position.z * 100.) as usize)
        .unwrap()
}

fn _get_center_from_vertices(vertices: &Vec<macroquad::models::Vertex>) -> Vec3 {
    vertices
        .iter()
        .map(|a| a.position)
        .reduce(|current, new| current + new)
        .unwrap()
        / (vertices.len() as f32)
}

fn load_model() -> Mesh {
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
    let vertices: Vec<macroquad::models::Vertex> = _get_vertices_from_mesh(mesh);

    Mesh {
        vertices,
        indices: mesh.indices.iter().map(|x| *x as u16).collect::<Vec<u16>>(),
        texture,
    }
}

#[macroquad::main("Flight Simulator")]
async fn main() {
    let mut plane: Plane = Plane::new(100., load_model());

    loop {
        let dt = get_frame_time();

        clear_background(LIGHTGRAY);
        draw_grid(50, 1.0, RED, GREEN);

        if is_key_down(KeyCode::Escape) {
            break;
        }

        let thrust = vec3(0., 0., 0.);
        let wind = vec3(0., 0., 0.);
        plane.update(dt, &thrust, &wind);
        plane.draw();

        set_camera(&plane.camera);

        next_frame().await
    }
}
