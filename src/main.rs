// use macroquad::models::Mesh;
use macroquad::prelude::*;

struct Plane {
    mass: f32,
    position: Vec3,
    velocity: Vec3,
    mesh: Mesh,
}

impl Plane {
    fn new(mass: f32, position: &Vec3, mesh: &Mesh) -> Self {
        Self {
            mass,
            position: *position,
            velocity: vec3(0., 0., 0.),
            mesh: *mesh,
        }
    }
    fn draw(&self) {}
    fn update(&mut self, dt: f32, thrust: &Vec3, wind: &Vec3) {}
}

#[macroquad::main("Flight Simulator")]
async fn main() {
    let mut camera = Camera3D {
        position: vec3(-20., 15., 0.),
        up: vec3(0., 1., 0.),
        target: vec3(0., 0., 0.),
        ..Default::default()
    };

    let mesh = Mesh::from("pass");
    let mut plane: Plane = Plane::new(100., &vec3(0., 0., 0.), &mesh);

    loop {
        let dt = get_frame_time();

        clear_background(WHITE);
        draw_grid(50, 1.0, RED, GREEN);

        camera.position += vec3(1., 1., 1.);
        set_camera(&camera);

        let thrust = vec3(0., 0., 0.);
        let wind = vec3(0., 0., 0.);
        plane.update(dt, &thrust, &wind);
        println!("{}", thrust);
        plane.draw();

        next_frame().await
    }
}
