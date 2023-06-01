use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use hexx::{HexLayout, PlaneMeshBuilder};

// Compute a bevy mesh from the layout
pub fn hexagonal_plane(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(hex_layout).facing(Vec3::Z).build();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs);
    mesh.set_indices(Some(Indices::U16(mesh_info.indices)));
    mesh
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum MaterialType {
    Plains,
    Mountain,
    Path,
    Goal,
    Spawn,
    Target,
    Enemy,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum MeshType {
    Hex,
    Enemy,
    Tower,
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DamageLevel {
    Low,
    Medium,
    High,
}

impl DamageLevel {
    pub fn get_level(damage: u32) -> Self {
        match damage {
            0..=3 => Self::Low,
            4..=7 => Self::Medium,
            _ => Self::High,
        }
    }
}
