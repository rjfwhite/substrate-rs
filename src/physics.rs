use glam::*;
use physx::prelude::*;
use specs::prelude::*;

use crate::common::*;

const PX_PHYSICS_VERSION: u32 = physx::version(4, 1, 1);

pub struct PhysicsSystem {
    foundation: Foundation,
    pub physics: Physics,
    pub scene: Box<Scene>,
}

impl PhysicsSystem {
    pub fn new() -> PhysicsSystem {
        let mut foundation = Foundation::new(PX_PHYSICS_VERSION);
        let mut physics = PhysicsBuilder::default()
            .load_extensions(false)
            .build(&mut foundation);

        let mut scene = physics.create_scene(
            SceneBuilder::default()
                .set_gravity(Vec3::new(0.0, -9.81, 0.0))
                .set_simulation_threading(SimulationThreadType::Dedicated(8)),
        );

        let material = physics.create_material(0.5, 0.5, 0.2);

        let ground_plane = unsafe { physics.create_plane(Vec3::new(0.0, 1.0, 0.0), 0.0, material) };
        scene.add_actor(ground_plane);

        return PhysicsSystem {
            foundation,
            physics,
            scene,
        };
    }
}

impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (Read<'a, DeltaTime>,
                       Entities<'a>,
                       WriteStorage<'a, Transform>,
                       ReadStorage<'a, BoxCollider>,
                       WriteStorage<'a, Rigidbody>);

    fn run(&mut self, (dt, entities, mut transform, collider, mut rigidbody): Self::SystemData) {

        self.scene.simulate(dt.0);
        self.scene.fetch_results(true).expect("error occured during simulation");

        for (e, t, c, r) in (&entities, &mut transform, &collider, &mut rigidbody).join() {
            let rb: &mut Rigidbody = r;

            match rb.0 {
                Some(body) => {
                    let model: Mat4 = self.scene.get_rigid_actor(body).expect("Yeee").get_global_pose();
                    t.0 = model;
                }
                None => {
                    let material = self.physics.create_material(0.5, 0.5, 0.2);
                    let sphere_geo = PhysicsGeometry::from(&ColliderDesc::Box(1.0, 1.0, 1.0));

                    let mut sphere_actor = unsafe {
                        self.physics.create_dynamic(
                            t.0,
                            sphere_geo.as_raw(), // todo: this should take the PhysicsGeometry straight.
                            material,
                            10.0,
                            Mat4::identity(),
                        )
                    };

                    sphere_actor.set_angular_damping(0.5);
                    let sphere_handle = self.scene.add_dynamic(sphere_actor);
                    rb.0 = Some(sphere_handle);
                }
            }
        }
    }
}