use glam::*;
use specs::{Component, VecStorage};
use physx::prelude::BodyHandle;

pub struct BoxCollider(pub Vec3);

impl Component for Transform {
    type Storage = VecStorage<Self>;
}

pub struct Transform(pub Mat4);

impl Component for BoxCollider {
    type Storage = VecStorage<Self>;
}

pub struct Rigidbody(pub Option<BodyHandle>);

impl Component for Rigidbody {
    type Storage = VecStorage<Self>;
}