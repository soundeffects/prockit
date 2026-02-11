use bevy::prelude::*;

pub struct RasterlessRenderPlugin;
impl Plugin for RasterlessRenderPlugin {
    fn build(&self, app: &mut App) {
        info!("RasterlessRenderPlugin loaded!");
    }
    fn finish(&self, app: &mut App) {
        info!("RasterlessRenderPlugin initialization complete!");
    }
}

//#[derive(AsBindGroup, Component, Clone)]
pub struct Ellipsoid {
    //#[texture(0)]
    //#[sampler(1)]
    //pub texture: Handle<Image>,
    pub radius: f32,
}
