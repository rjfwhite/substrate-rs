#[macro_use]
extern crate glam;
#[macro_use]
extern crate glium;
extern crate rand;
#[macro_use]
extern crate specs;

use glam::*;
use rand::Rng;
use specs::prelude::*;

use crate::common::*;
use std::time::{Duration, Instant};

mod colors;
mod common;
mod camera;
mod support;
mod physics;
mod rendering;
mod loader;

fn main() {
    let mut world = World::new();
    world.register::<Transform>();
    world.register::<BoxCollider>();
    world.register::<Rigidbody>();
    world.register::<PlaneCollider>();

    let mut randy = rand::thread_rng();

    for _ in 1..10000 {

        world.create_entity()
            .with(Transform(Mat4::from_translation(Vec3::new(randy.gen_range(-200.0, 200.0), randy.gen_range(20.0, 1000.0), randy.gen_range(-200.0, 200.0)))))
            .with(BoxCollider(Vec3::new(1.0, 1.0, 1.0)))
            .with(Rigidbody(None)).build();
    }

    world.create_entity()
        .with(Transform(Mat4::from_translation(Vec3::zero())))
        .with(PlaneCollider(Vec2::new(1000.0,  1000.0))).build();

    world.insert(DeltaTime(0.0));

    let mut physics_system = physics::PhysicsSystem::new();
    let mut rendering_system = rendering::RenderingSystem::new();

    let frame_time = 0.016;

    loop {
        let frame_start = Instant::now();
        world.insert(DeltaTime(frame_time));
        physics_system.run_now(&world);
        rendering_system.run_now(&world);

        let frame_simulation_time = Instant::now() - frame_start;
        let sleep_duration =  if frame_simulation_time < std::time::Duration::from_secs_f32(frame_time) {
            std::time::Duration::from_secs_f32(frame_time) - frame_simulation_time
        } else {
            std::time::Duration::from_secs_f32(frame_time)
        };
    }
}

