#[macro_use]
extern crate glam;
#[macro_use]
extern crate glium;
extern crate imgui;
extern crate physx;
extern crate rand;
#[macro_use]
extern crate specs;

use std::borrow::{Borrow, BorrowMut};
use std::intrinsics::transmute;

use glam::*;
use glium::{BackfaceCullingMode, Depth, DepthTest, glutin, IndexBuffer, Smooth, Surface, VertexFormat};
use glium::PolygonMode;
use glium::vertex::VertexBufferAny;
use imgui::*;
use imgui::sys::{ImGuiKey_C, ImGuiKey_DownArrow, ImGuiKey_UpArrow};
use physx::prelude::*;
use rand::Rng;
use specs::prelude::*;
use physx::user_data::UserData::RigidActor;

const PX_PHYSICS_VERSION: u32 = physx::version(4, 1, 1);

mod camera;
mod support;
mod loader;

struct BoxCollider(Vec3);

impl Component for Transform {
    type Storage = VecStorage<Self>;
}

struct Transform(Mat4);

impl Component for BoxCollider {
    type Storage = VecStorage<Self>;
}

struct Rigidbody(Option<BodyHandle>);

impl Component for Rigidbody {
    type Storage = VecStorage<Self>;
}

struct PhysicsSystem {
    foundation: Foundation,
    physics: Physics,
    scene: Box<Scene>,
}

impl PhysicsSystem {
    fn new() -> PhysicsSystem {
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
    type SystemData = (Entities<'a>,
                       WriteStorage<'a, Transform>,
                       ReadStorage<'a, BoxCollider>,
                       WriteStorage<'a, Rigidbody>);

    fn run(&mut self, (entities, mut transform, collider, mut rigidbody): Self::SystemData) {
        for (e, t, c, r) in (&entities, &mut transform, &collider, &mut rigidbody).join() {
            let rb: &mut Rigidbody = r;
            match rb.0 {
                Some(body) => {
                    let model: Mat4 = self.scene.get_rigid_actor(body).expect("Yeee").get_global_pose();
                    t.0 = model;
                },
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

                    println!("MADE ENTITY");
                }
            }
        }
    }
}

struct SysA {
    foo: i32
}

impl<'a> System<'a> for SysA {
    // These are the resources required for execution.
    // You can also define a struct and `#[derive(SystemData)]`,
    // see the `full` example.
    type SystemData = (Entities<'a>,
                       WriteStorage<'a, Transform>,
                       ReadStorage<'a, BoxCollider>);

    fn run(&mut self, (entities, mut transform, collider): Self::SystemData) {
        // The `.join()` combines multiple components,
        // so we only access those entities which have
        // both of them.

        // This joins the component storages for Position
        // and Velocity together; it's also possible to do this
        // in parallel using rayon's `ParallelIterator`s.
        // See `ParJoin` for more.
        for (entity, pos, vel) in (&entities, &mut transform, &collider).join() {
            println!("MOO {}", entity.id());
        }
    }
}

fn main() {
    let mut world = World::new();
    world.register::<Transform>();
    world.register::<BoxCollider>();
    world.register::<Rigidbody>();
    let a = world.create_entity().with(Transform(Mat4::from_translation(Vec3::new(0.0,20.0,0.0)))).with(BoxCollider(Vec3::one())).with(Rigidbody(None)).build();
    let b = world.create_entity().with(Transform(Mat4::from_translation(Vec3::new(0.0, 10.0, 0.0)))).with(BoxCollider(Vec3::one())).with(Rigidbody(None)).build();

    let mut system = SysA { foo: 50 };

    let mut dispatcher = DispatcherBuilder::new().with(SysA { foo: 900 }, "sys_a", &[]).build();
    dispatcher.dispatch(&mut world);


    system.run_now(&world);
    world.maintain();

    // for (e, t, c, r) in (&entities, &mut transform, &collider, &mut rigidbody).join()

    let mut physics_system = PhysicsSystem::new();

    physics_system.run_now(&world);

    // let mut foundation = Foundation::new(PX_PHYSICS_VERSION);

    // let mut physics = PhysicsBuilder::default()
    //     .load_extensions(false)
    //     .build(&mut foundation);
    //
    // let mut scene = physics.create_scene(
    //     SceneBuilder::default()
    //         .set_gravity(Vec3::new(0.0, -9.81, 0.0))
    //         .set_simulation_threading(SimulationThreadType::Dedicated(8)),
    // );
    //

    // let ground_plane = unsafe { physics.create_plane(Vec3::new(0.0, 1.0, 0.0), 0.0, material) };
    // scene.add_actor(ground_plane);
    let material = physics_system.physics.create_material(0.5, 0.5, 0.2);
    let sphere_geo = PhysicsGeometry::from(&ColliderDesc::Box(1.0, 1.0, 1.0));

    let mut bodies: Vec<BodyHandle> = Vec::new();

    let mut gen = rand::thread_rng();

    // for i in 1..100 {
    //     let mut sphere_actor = unsafe {
    //         physics_system.physics.create_dynamic(
    //             Mat4::from_translation(Vec3::new(gen.gen_range(-10.0, 10.0), 20.0 + gen.gen_range(-10.0, 100.0), gen.gen_range(-10.0, 10.0))),
    //             sphere_geo.as_raw(), // todo: this should take the PhysicsGeometry straight.
    //             material,
    //             10.0,
    //             Mat4::identity(),
    //         )
    //     };
    //
    //     sphere_actor.set_angular_damping(0.5);
    //     let sphere_handle = physics_system.scene.add_dynamic(sphere_actor);
    //     bodies.push(sphere_handle);
    // }

    let mut system = support::init(file!());

    let light_loc = [0.4, 1.0, 0.7];

    let shadow_map_size = 2048;
    let shadow_texture = glium::texture::DepthTexture2d::empty(&system.display, shadow_map_size, shadow_map_size).unwrap();
    let w = 50.0;
    let shadow_projection = Mat4::orthographic_rh_gl(-w, w, -w, w, -50.0, 100.0);
    let shadow_view = Mat4::look_at_rh(light_loc.into(), Vec3::zero(), Vec3::unit_y());


    let mut shadow_draw_params: glium::draw_parameters::DrawParameters = Default::default();
    shadow_draw_params.depth = glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLessOrEqual,
        write: true,
        ..Default::default()
    };
    shadow_draw_params.backface_culling = glium::BackfaceCullingMode::CullCounterClockwise;


    // let heights_over_time = (0..100)
    //     .map(|_| {
    //         scene.simulate(0.1);
    //         scene
    //             .fetch_results(true)
    //             .expect("error occured during simulation");
    //
    //         // in this case, we know the sphere still exists and is a
    //         // RigidActor-type so we can use the unchecked API
    //         unsafe { scene.get_rigid_actor_unchecked(&sphere_handle) }
    //             .get_global_position()
    //             .y() as i32
    //             - 10
    //      })
    //     .collect::<Vec<_>>();


    let program = glium::Program::from_source(
        &system.display,
        include_str!("../resources/shaders/diffuse.vert.glsl"),
        include_str!("../resources/shaders/diffuse.frag.glsl"),
        None,
    ).unwrap();

    let shadow_diffuse_program = glium::Program::from_source(
        &system.display,
        include_str!("../resources/shaders/diffuse-shadow.vert.glsl"),
        include_str!("../resources/shaders/diffuse-shadow.frag.glsl"),
        None,
    ).unwrap();

    let shadow_program = glium::Program::from_source(
        &system.display,
        include_str!("../resources/shaders/shadow.vert.glsl"),
        include_str!("../resources/shaders/shadow.frag.glsl"),
        None,
    ).unwrap();

    let image_program = glium::Program::from_source(
        &system.display,
        include_str!("../resources/shaders/image.vert.glsl"),
        include_str!("../resources/shaders/image.frag.glsl"),
        None,
    ).unwrap();

    let debug_vertex_buffer = glium::VertexBuffer::new(
        &system.display,
        &[
            DebugVertex::new([0.25, -1.0], [0.0, 0.0]),
            DebugVertex::new([0.25, -0.25], [0.0, 1.0]),
            DebugVertex::new([1.0, -0.25], [1.0, 1.0]),
            DebugVertex::new([1.0, -1.0], [1.0, 0.0]),
        ],
    ).unwrap();
    let debug_index_buffer = glium::IndexBuffer::new(
        &system.display,
        glium::index::PrimitiveType::TrianglesList,
        &[0u16, 1, 2, 0, 2, 3],
    ).unwrap();

    let cube = loader::load_wavefront(&system.display, include_bytes!("../resources/models/cube.obj"));
    let mut camera = camera::Camera::new(Vec3::new(0.0, 2.0, 0.0));

    let mut t: f32 = 0.0;

    system.main_loop(move |_, ui, target, display| {
        let dt = ui.io().delta_time;

        t += dt;

        camera.update_from_io(&ui.io());



        physics_system.scene.simulate(dt);

        physics_system.scene.fetch_results(true).expect("error occured during simulation");


        physics_system.run_now(&world);


        target.clear_color_and_depth((0.01, 0.01, 0.01, 0.8), 1.0);

        let (width, height) = display.get_framebuffer_dimensions();
        let aspect_ratio = width as f32 / height as f32;
        let projection = Mat4::perspective_rh_gl(3.3141 / 4.0, aspect_ratio, 0.1, 1000.0);

        let view = camera.transform();

        let floor = Mat4::from_translation(-2.0 * Vec3::unit_y()) * Mat4::from_scale(Vec3::new(200.0, 2.0, 200.0));


        //draw shadow
        {
            let mut shadow_target = glium::framebuffer::SimpleFrameBuffer::depth_only(display, &shadow_texture).unwrap();
            shadow_target.clear_color(1.0, 1.0, 1.0, 1.0);
            shadow_target.clear_depth(1.0);

            for i in 0..bodies.len() {
                let model: Mat4 = physics_system.scene.get_rigid_actor(bodies[i]).expect("Yeee").get_global_pose();
                let depth_mvp = shadow_projection * shadow_view * model;
                let uniforms = uniform! {
                    depth_mvp: depth_mvp.to_cols_array_2d(),
                };
                shadow_target.draw(
                    &cube,
                    &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                    &shadow_program,
                    &uniforms,
                    &shadow_draw_params,
                ).unwrap();
            }
        }


        let bias_matrix = Mat4::from_cols_array_2d(&[
            [0.5, 0.0, 0.0, 0.0f32],
            [0.0, 0.5, 0.0, 0.0f32],
            [0.0, 0.0, 0.5, 0.0f32],
            [0.5, 0.5, 0.5, 1.0f32],
        ]);

        let mut draw_params: glium::draw_parameters::DrawParameters = Default::default();
        draw_params.depth = glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLessOrEqual,
            write: true,
            ..Default::default()
        };
        draw_params.backface_culling = glium::BackfaceCullingMode::CullClockwise;
        draw_params.blend = glium::Blend::alpha_blending();



        for (t) in (world.read_component::<Transform>()).join() {
            let pos : Mat4 = t.0;

            let bias_depth_mvp = bias_matrix * shadow_projection * shadow_view * pos;

            let uniforms = uniform! {
                light_loc: light_loc,
                projection: projection.to_cols_array_2d(),
                view: view.to_cols_array_2d(),
                model: pos.to_cols_array_2d(),
                paint: [0.0, 1.0, 0.0f32],
                depth_bias_mvp: bias_depth_mvp.to_cols_array_2d(),
                shadow_map: glium::uniforms::Sampler::new(&shadow_texture)
					.magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
					.minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .depth_texture_comparison(Some(glium::uniforms::DepthTextureComparison::LessOrEqual)),
            };

            target.draw(
                &cube,
                &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &shadow_diffuse_program,
                &uniforms,
                &draw_params,
            ).unwrap();

            draw_geometry(target, &cube, projection, view, pos, Vec3::new(0.0, 1.0, 0.0), true, &program);
        }


        // for i in 0..bodies.len() {
        //
        //
        //
        //     let pos: Mat4 = physics_system.scene.get_rigid_actor(bodies[i]).expect("Yeee").get_global_pose();
        //
        //
        // }

        // draw_geometry(target, &cube, projection, view, floor, Vec3::new(0.1, 0.1, 0.1), false,&program);

        let bias_depth_mvp = bias_matrix * shadow_projection * shadow_view * floor;

        let uniforms = uniform! {
                light_loc: light_loc,
                projection: projection.to_cols_array_2d(),
                view: view.to_cols_array_2d(),
                model: floor.to_cols_array_2d(),
                paint: [0.1, 0.1, 0.1f32],
                depth_bias_mvp: bias_depth_mvp.to_cols_array_2d(),
                shadow_map: glium::uniforms::Sampler::new(&shadow_texture)
					.magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
					.minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .depth_texture_comparison(Some(glium::uniforms::DepthTextureComparison::LessOrEqual)),
            };

        target.draw(
            &cube,
            &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
            &shadow_diffuse_program,
            &uniforms,
            &draw_params,
        ).unwrap();


        // shadow debug
        // {
        //     let uniforms = uniform! {
        //         tex: glium::uniforms::Sampler::new(&shadow_texture)
        //             .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        //             .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
        //     };
        //     target.clear_depth(1.0);
        //     target
        //         .draw(
        //             &debug_vertex_buffer,
        //             &debug_index_buffer,
        //             &image_program,
        //             &uniforms,
        //             &Default::default(),
        //         )
        //         .unwrap();
        // }


        Window::new(im_str!("Hello world"))
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("Hello world!"));
                ui.text(im_str!("This...is...imgui-rs!"));


                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));

                ui.text(format!(
                    "Camera Position: ({:.1},{:.1},{:.1})",
                    camera.position.x(), camera.position.y(), camera.position.z()
                ));

                ui.text(format!(
                    "Azimuth Pitch: ({:.1},{:.1})",
                    camera.azimuth, camera.pitch,
                ));
            });
    });

    // unsafe {
    //     scene.release();
    // }
    // drop(physics_system.physics);
    // physics_system.foundation.release();
}

fn draw_geometry(target: &mut glium::Frame,
                 vertex_buffer: &VertexBufferAny,
                 projection: Mat4,
                 view: Mat4,
                 model: Mat4,
                 color: Vec3,
                 wireframe: bool,
                 program: &glium::Program) {
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let mut params = glium::DrawParameters::default();
    params.line_width = Option::Some(3.0);
    params.polygon_mode = PolygonMode::Fill;
    params.backface_culling = BackfaceCullingMode::CullClockwise;
    params.depth.write = true;
    params.depth.test = DepthTest::IfLessOrEqual;

    let c: [f32; 3] = color.into();

    if !wireframe {
        target.draw(vertex_buffer, &indices, program, &uniform! {
             projection: projection.to_cols_array_2d(),
             view: view.to_cols_array_2d(),
             model: model.to_cols_array_2d(),
             paint:c
        }, &params).unwrap();
    }

    if wireframe {
        params.polygon_mode = PolygonMode::Line;
        params.depth.test = DepthTest::IfLessOrEqual;

        target.draw(vertex_buffer, &indices, program, &uniform! {
             projection: projection.to_cols_array_2d(),
             view: view.to_cols_array_2d(),
             model: model.to_cols_array_2d(),
             paint: [0.0, 0.0, 0.0f32]
        }, &params).unwrap();
    }
}


#[derive(Clone, Copy, Debug)]
struct DebugVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(DebugVertex, position, tex_coords);
impl DebugVertex {
    pub fn new(position: [f32; 2], tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            tex_coords,
        }
    }
}