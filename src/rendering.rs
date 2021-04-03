use glam::*;
use glium::{BackfaceCullingMode, DepthTest, PolygonMode, Program, Surface};
use glium::vertex::VertexBufferAny;
use imgui::*;
use rand::Rng;
use specs::*;
use stopwatch::Stopwatch;
use winit::event_loop::ControlFlow;
use winit::platform::desktop::EventLoopExtDesktop;

use crate::{colors, loader};
use crate::common::*;

pub struct RenderingSystem<'a> {
    system: crate::support::System,
    camera: crate::camera::Camera,
    diffuse_program: Program,
    instanced_diffuse_program: Program,
    shadow_diffuse_program: Program,
    instanced_shadow_diffuse_program: Program,
    shadow_program: Program,
    instanced_shadow_program: Program,
    image_program: Program,
    solid_program: Program,
    cube: VertexBufferAny,
    plane: VertexBufferAny,
    shadow_texture: glium::texture::DepthTexture2d,
    shadow_draw_params: glium::draw_parameters::DrawParameters<'a>,
    shadow_projection: Mat4,
    shadow_view: Mat4,
    light_loc: Vec3,
}

impl<'a> RenderingSystem<'a> {
    pub fn new() -> RenderingSystem<'a> {
        let system = crate::support::init(file!());

        let diffuse_program = glium::Program::from_source(
            &system.display,
            include_str!("../resources/shaders/diffuse.vert.glsl"),
            include_str!("../resources/shaders/diffuse.frag.glsl"),
            None,
        ).unwrap();

        let instanced_diffuse_program = glium::Program::from_source(
            &system.display,
            include_str!("../resources/shaders/diffuse-instanced.vert.glsl"),
            include_str!("../resources/shaders/diffuse-instanced.frag.glsl"),
            None,
        ).unwrap();

        let shadow_diffuse_program = glium::Program::from_source(
            &system.display,
            include_str!("../resources/shaders/diffuse-shadow.vert.glsl"),
            include_str!("../resources/shaders/diffuse-shadow.frag.glsl"),
            None,
        ).unwrap();

        let instanced_shadow_diffuse_program = glium::Program::from_source(
            &system.display,
            include_str!("../resources/shaders/diffuse-shadow-instanced.vert.glsl"),
            include_str!("../resources/shaders/diffuse-shadow-instanced.frag.glsl"),
            None,
        ).unwrap();

        let shadow_program = glium::Program::from_source(
            &system.display,
            include_str!("../resources/shaders/shadow.vert.glsl"),
            include_str!("../resources/shaders/shadow.frag.glsl"),
            None,
        ).unwrap();

        let instanced_shadow_program = glium::Program::from_source(
            &system.display,
            include_str!("../resources/shaders/shadow-instanced.vert.glsl"),
            include_str!("../resources/shaders/shadow-instanced.frag.glsl"),
            None,
        ).unwrap();

        let image_program = glium::Program::from_source(
            &system.display,
            include_str!("../resources/shaders/image.vert.glsl"),
            include_str!("../resources/shaders/image.frag.glsl"),
            None,
        ).unwrap();

        let solid_program = glium::Program::from_source(
            &system.display,
            include_str!("../resources/shaders/solid.vert.glsl"),
            include_str!("../resources/shaders/solid.frag.glsl"),
            None,
        ).unwrap();

        let light_loc = [0.4, 1.0, 0.7];
        let shadow_map_size = 10000;
        let shadow_texture = glium::texture::DepthTexture2d::empty(&system.display, shadow_map_size, shadow_map_size).unwrap();
        let w = 100.0;
        let shadow_projection = Mat4::orthographic_rh_gl(-w, w, -w, w, -200.0, 600.0);
        let shadow_view = Mat4::look_at_rh(light_loc.into(), Vec3::zero(), Vec3::unit_y());

        let mut shadow_draw_params: glium::draw_parameters::DrawParameters = Default::default();
        shadow_draw_params.depth = glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLessOrEqual,
            write: true,
            ..Default::default()
        };
        shadow_draw_params.backface_culling = glium::BackfaceCullingMode::CullCounterClockwise;

        let cube = loader::load_wavefront(&system.display, include_bytes!("../resources/models/cube.obj"));
        let plane = loader::load_wavefront(&system.display, include_bytes!("../resources/models/plane.obj"));

        RenderingSystem {
            system,
            camera: crate::camera::Camera::new(Vec3::zero()),
            diffuse_program,
            instanced_diffuse_program,
            shadow_diffuse_program,
            instanced_shadow_diffuse_program,
            shadow_program,
            instanced_shadow_program,
            image_program,
            solid_program,
            cube,
            plane,
            shadow_texture,
            shadow_draw_params,
            shadow_projection,
            shadow_view,
            light_loc: light_loc.into(),
        }
    }

    fn draw_shadow<'b>(&self, transforms: &ReadStorage<'b, Transform>,
                       boxes: &ReadStorage<'b, BoxCollider>,
                       planes: &ReadStorage<'b, PlaneCollider>) {
        let mut shadow_target = glium::framebuffer::SimpleFrameBuffer::depth_only(&self.system.display, &self.shadow_texture).unwrap();
        shadow_target.clear_color(1.0, 1.0, 1.0, 1.0);
        shadow_target.clear_depth(1.0);

        let mut box_transforms: Vec<Mat4> = Vec::new();

        for (transform, boxc) in (transforms, boxes).join() {
            let model: Mat4 = transform.0 * Mat4::from_scale(boxc.0);
            box_transforms.push(model);
        }

        let mut per_instance = {
            #[derive(Copy, Clone)]
            struct Attr {
                model: [[f32; 4]; 4]
            }

            implement_vertex!(Attr, model);

            let data = box_transforms.iter().map(|model| {
                Attr {
                    model: model.to_cols_array_2d()
                }
            }).collect::<Vec<_>>();

            glium::vertex::VertexBuffer::dynamic(&self.system.display, &data).unwrap()
        };

        {
            let uniforms = uniform! {
                    projection: self.shadow_projection.to_cols_array_2d(),
                    view: self.shadow_view.to_cols_array_2d(),
                };

            shadow_target.draw(
                (&self.cube, per_instance.per_instance().unwrap()),
                &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &self.instanced_shadow_program,
                &uniforms,
                &self.shadow_draw_params,
            ).unwrap();
        }
    }

    fn draw_geometry<'b>(&self, target: &mut glium::Frame, transforms: &ReadStorage<'b, Transform>,
                         boxes: &ReadStorage<'b, BoxCollider>,
                         planes: &ReadStorage<'b, PlaneCollider>) {
        let aspect_ratio = {
            let (width, height) = self.system.display.get_framebuffer_dimensions();
            width as f32 / height as f32
        };
        let projection = Mat4::perspective_rh_gl(3.3141 / 4.0, aspect_ratio, 0.1, 1000.0);
        let view = self.camera.transform();

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

        let mut box_transforms: Vec<Mat4> = Vec::new();

        for (transform, boxc) in (transforms, boxes).join() {
            let pos: Mat4 = transform.0 * Mat4::from_scale(boxc.0);

            let mut transformed = projection * view * (transform.0 * Vec4::new(0.0, 0.0, 0.0, 1.0));
            transformed /= transformed.w();

            if transformed.z() > -0.1 && transformed.x() > -1.1 && transformed.x() < 1.1 && transformed.y() > -1.1 && transformed.y() < 1.1 {
                box_transforms.push(pos);
            }
        }

        for (transform, plane) in (transforms, planes).join() {
            let pos: Mat4 = transform.0 * Mat4::from_scale(Vec3::new(plane.0.x(), 1.0, plane.0.y()));
            {
                let bias_depth_mvp = bias_matrix * self.shadow_projection * self.shadow_view * pos;
                let uniforms = uniform! {
                    light_loc: [self.light_loc.x(), self.light_loc.y(), self.light_loc.z()],
                    projection: projection.to_cols_array_2d(),
                    view: view.to_cols_array_2d(),
                    model: pos.to_cols_array_2d(),
                    paint: [0.3, 0.3, 0.3f32],
                    depth_bias_mvp: bias_depth_mvp.to_cols_array_2d(),
                    shadow_map: glium::uniforms::Sampler::new(&self.shadow_texture)
        				.magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        				.minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                        .depth_texture_comparison(Some(glium::uniforms::DepthTextureComparison::LessOrEqual)),
                };

                target.draw(
                    &self.plane,
                    &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                    &self.shadow_diffuse_program,
                    &uniforms,
                    &draw_params,
                ).unwrap();
            }

            {
                let bias_depth_mvp = bias_matrix * self.shadow_projection * self.shadow_view * pos;
                let uniforms = uniform! {
                    projection: projection.to_cols_array_2d(),
                    view: view.to_cols_array_2d(),
                    model: pos.to_cols_array_2d(),
                    paint: [0.0, 0.0, 0.0f32]
                };

                draw_params.polygon_mode = PolygonMode::Line;
                draw_params.line_width = Some(2.0);

                target.draw(
                    &self.plane,
                    &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                    &self.shadow_diffuse_program,
                    &uniforms,
                    &draw_params,
                ).unwrap();

                draw_params.polygon_mode = PolygonMode::Fill;
            }
        }

        let mut randy = rand::thread_rng();

        let mut per_instance = {
            #[derive(Copy, Clone)]
            struct Attr {
                model: [[f32; 4]; 4]
            }

            implement_vertex!(Attr, model);

            let data = box_transforms.iter().map(|model| {
                Attr {
                    model: model.to_cols_array_2d()
                }
            }).collect::<Vec<_>>();

            glium::vertex::VertexBuffer::dynamic(&self.system.display, &data).unwrap()
        };


        {
            let bias_depth_mvp = bias_matrix * self.shadow_projection * self.shadow_view;
            let uniforms = uniform! {
                    light_loc: [self.light_loc.x(), self.light_loc.y(), self.light_loc.z()],
                    projection: projection.to_cols_array_2d(),
                    view: view.to_cols_array_2d(),
                    paint: [0.0, 1.0, 0.0f32],
                    depth_bias_mvp: bias_depth_mvp.to_cols_array_2d(),
                    shadow_map: glium::uniforms::Sampler::new(&self.shadow_texture)
        				.magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        				.minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                        .depth_texture_comparison(Some(glium::uniforms::DepthTextureComparison::LessOrEqual)),
                };

            target.draw(
                (&self.cube, per_instance.per_instance().unwrap()),
                &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &self.instanced_shadow_diffuse_program,
                &uniforms,
                &draw_params,
            ).unwrap();
        }

        {
            let uniforms = uniform! {
                    projection: projection.to_cols_array_2d(),
                    view: view.to_cols_array_2d(),
                    paint: [0.0, 0.0, 0.0f32],
                };

            draw_params.polygon_mode = PolygonMode::Line;
            target.draw(
                (&self.cube, per_instance.per_instance().unwrap()),
                &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &self.instanced_diffuse_program,
                &uniforms,
                &draw_params,
            ).unwrap();
            draw_params.polygon_mode = PolygonMode::Fill;
        }
    }

    fn draw_debug_shadow_map(&self, target: &mut glium::Frame) {
        let debug_vertex_buffer = glium::VertexBuffer::new(
            &self.system.display,
            &[
                DebugVertex::new([0.25, -1.0], [0.0, 0.0]),
                DebugVertex::new([0.25, -0.25], [0.0, 1.0]),
                DebugVertex::new([1.0, -0.25], [1.0, 1.0]),
                DebugVertex::new([1.0, -1.0], [1.0, 0.0]),
            ],
        ).unwrap();
        let debug_index_buffer = glium::IndexBuffer::new(
            &self.system.display,
            glium::index::PrimitiveType::TrianglesList,
            &[0u16, 1, 2, 0, 2, 3],
        ).unwrap();

        let uniforms = uniform! {
                tex: glium::uniforms::Sampler::new(&self.shadow_texture)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
            };
        target.clear_depth(1.0);
        target.draw(
            &debug_vertex_buffer,
            &debug_index_buffer,
            &self.image_program,
            &uniforms,
            &Default::default(),
        )
            .unwrap();
    }
}

impl<'a> System<'a> for RenderingSystem<'_> {
    type SystemData = (Read<'a, DeltaTime>,
                       ReadStorage<'a, Transform>,
                       ReadStorage<'a, BoxCollider>,
                       ReadStorage<'a, PlaneCollider>);

    fn run(&mut self, (dt, transforms, boxes, planes): Self::SystemData) {
        let sw = Stopwatch::start_new();

        {
            let display = &self.system.display;
            let imgui = &mut self.system.imgui;
            let platform = &mut self.system.platform;

            use crate::glium::glutin::platform::desktop::EventLoopExtDesktop;
            self.system.event_loop.run_return(|event, _, control_flow| {
                match event {
                    event => {
                        let gl_window = display.gl_window();
                        platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
                    }
                }

                *control_flow = ControlFlow::Exit;
            });
        }

        self.camera.update_from_io(self.system.imgui.io());

        let mut target = self.system.display.draw();
        target.clear_color_and_depth((0.01, 0.01, 0.01, 1.0), 1.0);

        self.draw_shadow(&transforms, &boxes, &planes);
        self.draw_geometry(&mut target, &transforms, &boxes, &planes);

        let mut ui = self.system.imgui.frame();

        {
            let camera = &self.camera;

            Window::new(im_str!("Hello world"))
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(&ui, || {
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
        }

        let gl_window = self.system.display.gl_window();

        self.system.platform.prepare_render(&ui, gl_window.window());
        let draw_data = ui.render();
        self.system.renderer
            .render(&mut target, draw_data)
            .expect("Rendering failed");
        target.finish().expect("Failed to swap buffers");

        println!("Rendering took {}ms", sw.elapsed_ms());
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