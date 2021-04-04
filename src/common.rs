use glam::*;
use physx::prelude::BodyHandle;
use specs::{Component, VecStorage};

pub enum Mesh {
    Cube,
    Plane,
    Sphere,
}

#[derive(Default)]
pub struct DeltaTime(pub f32);

pub struct BoxCollider(pub Vec3);

pub struct PlaneCollider(pub Vec2);

impl Component for PlaneCollider {
    type Storage = VecStorage<Self>;
}

pub struct Transform(pub Mat4);

impl Component for Transform {
    type Storage = VecStorage<Self>;
}

pub struct MeshRenderer(pub Mesh);

impl Component for MeshRenderer {
    type Storage = VecStorage<Self>;
}

impl Component for BoxCollider {
    type Storage = VecStorage<Self>;
}

pub struct Rigidbody(pub Option<BodyHandle>);

impl Component for Rigidbody {
    type Storage = VecStorage<Self>;
}
