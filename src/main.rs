#[macro_use]
extern crate glam;
#[macro_use]
extern crate glium;
extern crate imgui;

use glam::*;
use glium::{BackfaceCullingMode, Depth, DepthTest, glutin, IndexBuffer, Surface, VertexFormat, Smooth};
use glium::PolygonMode;
use glium::vertex::VertexBufferAny;
use imgui::*;
use imgui::sys::{ImGuiKey_C, ImGuiKey_DownArrow, ImGuiKey_UpArrow};

mod camera;
mod support;
mod loader;

fn main() {
    let mut system = support::init(file!());

    let program = glium::Program::from_source(
        &system.display,
        include_str!("../resources/shaders/diffuse.vert.glsl"),
        include_str!("../resources/shaders/diffuse.frag.glsl"),
        None,
    ).unwrap();

    let cube = loader::load_wavefront(&system.display, include_bytes!("../resources/models/cube.obj"));
    let mut camera = camera::Camera::new(Vec3::zero());

    system.main_loop(move |_, ui, target, display| {
        let dt = ui.io().delta_time;
        camera.update_from_io(&ui.io());
        target.clear_color_and_depth((0.01, 0.01, 0.01, 0.8), 1.0);

        let (width, height) = display.get_framebuffer_dimensions();
        let aspect_ratio = width as f32 / height as f32;
        let projection = Mat4::perspective_rh_gl(3.3141 / 4.0, aspect_ratio, 0.1, 100.0);

        let view = camera.transform();
        let model = Mat4::from_scale(Vec3::new(1.0, 1.0, 1.0));

        draw_geometry(target, &cube, projection, view, model, Vec3::new(0.0, 1.0, 0.0), &program);

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
}

fn draw_geometry(target: &mut glium::Frame,
                 vertex_buffer: &VertexBufferAny,
                 projection: Mat4,
                 view: Mat4,
                 model: Mat4,
                 color: Vec3,
                 program: &glium::Program) {
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    let mut params = glium::DrawParameters::default();
    params.line_width = Option::Some(3.0);
    params.polygon_mode = PolygonMode::Fill;
    params.backface_culling = BackfaceCullingMode::CullClockwise;
    params.depth.write = true;
    params.depth.test = DepthTest::IfLessOrEqual;
    params.multisampling = true;
    params.smooth = Some(Smooth::Nicest);

    let c: [f32; 3] = color.into();

    target.draw(vertex_buffer, &indices, program, &uniform! {
             projection: projection.to_cols_array_2d(),
             view: view.to_cols_array_2d(),
             model: model.to_cols_array_2d(),
             paint:c
        }, &params).unwrap();

    params.polygon_mode = PolygonMode::Line;
    params.depth.test = DepthTest::Overwrite;

    target.draw(vertex_buffer, &indices, program, &uniform! {
             projection: projection.to_cols_array_2d(),
             view: view.to_cols_array_2d(),
             model: model.to_cols_array_2d(),
             paint: [0.0, 0.0, 0.0f32]
        }, &params).unwrap();
}