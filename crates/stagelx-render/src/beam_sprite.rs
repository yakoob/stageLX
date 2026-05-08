use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError},
    shader::ShaderRef,
    mesh::MeshVertexBufferLayoutRef,
};

// ─── BeamSpriteMaterial ───────────────────────────────────────────────────────

/// Camera-facing billboard material for Tier 0 beam LOD.
///
/// Bindings:
///   0  color        — linear RGBA premultiplied by dimmer/strobe
///   1  sprite_params — x=rotation_radians  y=falloff_k  z=inner_radius  w=unused
///   2  gobo         — gobo mask texture
///   3  gobo sampler — auto
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BeamSpriteMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(1)]
    pub sprite_params: Vec4,
    #[texture(2)]
    #[sampler(3)]
    pub gobo: Handle<Image>,
}

impl Material for BeamSpriteMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/beam_sprite.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

// ─── Components ───────────────────────────────────────────────────────────────

/// Marks a beam sprite entity (Tier 0 billboard).
#[derive(Component)]
pub struct BeamSprite {
    pub id: stagelx_core::types::FixtureId,
}

// ─── Mesh ─────────────────────────────────────────────────────────────────────

/// Build a camera-facing quad mesh for the sprite.
pub fn build_beam_sprite_quad(size: f32) -> Mesh {
    let h = size * 0.5;
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-h, -h, 0.0],
            [ h, -h, 0.0],
            [ h,  h, 0.0],
            [-h,  h, 0.0],
        ],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ],
    );
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}
