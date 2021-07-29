use macroquad::prelude::*;

struct Plane {
    head: Vec3,
    center: Vec3,
    right_wing_tip: Vec3,
    velocity: Vec3,
    angular_velocity: Vec3,
    acceleration: Vec3,
    mesh: macroquad::models::Mesh,
    camera: Camera3D,
}

impl Plane {
    fn new(mesh: Mesh) -> Self {
        let mut plane = Self {
            head: _get_head_from_vertices(&mesh.vertices),
            center: _get_center_from_vertices(&mesh.vertices),
            right_wing_tip: _get_right_wing_tip_from_vertices(&mesh.vertices),
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            mesh,
            camera: Camera3D {
                ..Default::default()
            },
        };
        plane.camera.position = plane.up() * 2. + plane.forward() * -2.;
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
        self.center = _get_center_from_vertices(&self.mesh.vertices);
        let temp = _get_camera_vectors(&self);
        self.camera.position = temp.0;
        self.camera.target = temp.1;
        self.camera.up = temp.2;
    }

    fn rotate_by_axis_angle(&mut self, axis: &Vec3, angle: f32) {
        let rotation_matrix = glam::Mat3::from_axis_angle(*axis, angle);
        for vertex in &mut self.mesh.vertices {
            vertex.position = rotation_matrix.mul_vec3(vertex.position - self.center) + self.center;
        }
        self.head = rotation_matrix.mul_vec3(self.head - self.center) + self.center;
        self.right_wing_tip =
            rotation_matrix.mul_vec3(self.right_wing_tip - self.center) + self.center;
        self.center = _get_center_from_vertices(&self.mesh.vertices);
        let temp = _get_camera_vectors(&self);
        self.camera.position = temp.0;
        self.camera.target = temp.1;
        self.camera.up = temp.2;
    }

    fn rotate_by(&mut self, rotate_vector: Vec3) {
        self.rotate_by_axis_angle(&self.forward(), rotate_vector.x);
        self.rotate_by_axis_angle(&self.up(), rotate_vector.y);
        self.rotate_by_axis_angle(&self.right(), rotate_vector.z);
    }

    fn get_aerodynamic_force_and_torque(&self) -> (Vec3, Vec3) {
        let mut res_force = Vec3::ZERO;
        let mut res_torque = Vec3::ZERO;
        for [i, j, k] in self.mesh.indices.chunks_exact(3).map(|index_group| {
            [
                self.mesh.vertices[index_group[0] as usize].position - self.center,
                self.mesh.vertices[index_group[1] as usize].position - self.center,
                self.mesh.vertices[index_group[2] as usize].position - self.center,
            ]
        }) {
            let side1 = i - j;
            let side2 = j - k;
            let centroid = (i + j + k) / 3.;

            let normal = side1.cross(side2).normalize();
            let area = (side1.cross(side2) * 0.5).length();
            let tangential_velocity = self.velocity - (self.velocity.dot(normal)) * normal;
            let force = tangential_velocity.length().powi(2) * area * normal;
            let torque = -centroid.cross(force) * centroid.length_recip().powi(2);

            res_torque += torque;
            res_force += force;
        }
        (
            res_force * 0.1,
            Vec3::ZERO,
            // mat3(self.forward(), self.up(), self.right()).mul_vec3(res_torque) * 0.001,
        )
    }

    fn update(&mut self, dt: f32, thrust: &Vec3, torque: &Vec3, wind: &Vec3, gravity: &Vec3) {
        let (aerodynamic_force, aerodynamic_torque) = self.get_aerodynamic_force_and_torque();
        self.acceleration = aerodynamic_force + *thrust + *wind + *gravity;
        let angular_acceleration = aerodynamic_torque + *torque;
        self.angular_velocity += angular_acceleration * dt;
        self.velocity += self.acceleration * dt;
        if self.center.y <= 1. && self.velocity.y < 0. {
            self.velocity.y = 0.;
        }
        if self.center.y >= 49. && self.velocity.y > 0. {
            self.velocity.y = 0.;
        }
        if self.center.x >= 500. {
            self.translate_by(Vec3::X * -1000.);
        }
        if self.center.x <= -500. {
            self.translate_by(Vec3::X * 1000.);
        }
        if self.center.z >= 500. {
            self.translate_by(Vec3::Z * -1000.);
        }
        if self.center.z <= -500. {
            self.translate_by(Vec3::Z * 1000.);
        }
        self.velocity = self.velocity.clamp_length_max(50.);
        self.angular_velocity = self.angular_velocity.clamp_length_max(5.);
        self.translate_by(self.velocity * dt);
        self.rotate_by(self.angular_velocity * dt);
        self.angular_velocity *= 0.99;
    }
    fn up(&self) -> Vec3 {
        (self.right_wing_tip - self.center)
            .cross(self.head - self.center)
            .normalize()
    }
    fn forward(&self) -> Vec3 {
        (self.head - self.center).normalize()
    }
    fn backward(&self) -> Vec3 {
        -self.forward()
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

fn _get_camera_vectors(plane: &Plane) -> (Vec3, Vec3, Vec3) {
    (
        plane.center + plane.up() * 2. + plane.forward() * -2.,
        plane.center + plane.forward() * 3.,
        plane.up(),
    )
}

fn load_model() -> Mesh {
    let (models, _materials) = tobj::load_obj(
        "assets/plane.obj",
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
    )
    .expect("Failed to load");
    let texture = Some(Texture2D::from_file_with_format(
        include_bytes!("../assets/texture.png"),
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

fn pretty_vector(vector: &Vec3) -> String {
    format!("[{:.2}, {:.2}, {:.2}]", vector.x, vector.y, vector.z)
}

#[macroquad::main("Flight Simulator")]
async fn main() {
    let mut plane: Plane = Plane::new(load_model());

    let gravity = Vec3::Y * -0.5;
    let wind = Vec3::ZERO;

    loop {
        let dt = get_frame_time();

        clear_background(LIGHTGRAY);
        draw_grid(1000, 2.0, RED, GREEN);

        let mut thrust = Vec3::ZERO;
        let mut torque = Vec3::ZERO;

        if is_key_down(KeyCode::Escape) {
            break;
        }
        if is_key_down(KeyCode::W) {
            thrust += plane.forward() * 2.;
        }
        if is_key_down(KeyCode::S) {
            thrust += plane.backward() * 0.5;
        }
        if is_key_down(KeyCode::Left) {
            torque.y = 0.1;
        }
        if is_key_down(KeyCode::Right) {
            torque.y = -0.1;
        }
        if is_key_down(KeyCode::Up) {
            torque.z = 0.1;
        }
        if is_key_down(KeyCode::Down) {
            torque.z = -0.1;
        }
        if is_key_down(KeyCode::A) {
            torque.x = -0.1;
        }
        if is_key_down(KeyCode::D) {
            torque.x = 0.1;
        }

        plane.update(dt, &thrust, &torque, &wind, &gravity);

        set_default_camera();
        draw_text(
            &("Position    : ".to_owned() + &pretty_vector(&plane.center)),
            20.,
            30.,
            20.,
            RED,
        );
        draw_text(
            &("Velocity    : ".to_owned() + &pretty_vector(&plane.velocity)),
            20.,
            50.,
            20.,
            RED,
        );
        draw_text(
            &("Acceleration: ".to_owned() + &pretty_vector(&plane.acceleration)),
            20.,
            70.,
            20.,
            RED,
        );
        draw_text(
            &("Angular Vel : ".to_owned() + &pretty_vector(&plane.angular_velocity)),
            20.,
            90.,
            20.,
            RED,
        );

        set_camera(&plane.camera);
        plane.draw();
        next_frame().await
    }
}
