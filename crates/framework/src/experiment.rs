use super::{Names, Provides};
use bevy::{ecs::lifecycle::HookContext, prelude::*};
use std::marker::PhantomData;

pub trait Provider {
    fn provides() -> Provides;
}

#[derive(Component, Default)]
pub struct ProceduralNode {
    provides: Provides,
}

impl ProceduralNode {
    pub(crate) fn set_provides(&mut self, provides: Provides) {
        self.provides = provides;
    }
}

#[derive(Component)]
pub struct Volume<T, const N: usize> {
    voxels: [[[T; N]; N]; N],
}

impl<T, const N: usize> Volume<T, N> {
    pub fn coordinates(&self, x: usize, y: usize, z: usize) -> &T {
        &self.voxels[x][y][z]
    }
}

impl<T: Send + Sync + Copy + 'static, const N: usize> Provider for Volume<T, N> {
    fn provides() -> Provides {
        let mut provides = Provides::new();
        provides.add_4(
            Self::coordinates,
            Names::from("coordinates"),
            Names::from("self"),
            Names::from("x"),
            Names::from("y"),
            Names::from("z"),
        );
        provides
    }
}

pub struct ProceduralNodeHandler<T> {
    phantom_data: PhantomData<T>,
}

impl<T: Component + Provider> Plugin for ProceduralNodeHandler<T> {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        world.register_required_components::<T, ProceduralNode>();
        world
            .register_component_hooks::<T>()
            .on_add(|mut world, HookContext { entity, .. }| {
                // let Some(provides) = world
                //     .get::<T>(entity)
                //     .and_then(|component| Some(T::provides()))
                // else {
                //     return;
                // };

                world
                    .get_mut::<ProceduralNode>(entity)
                    .map(|mut node| node.set_provides(T::provides()));
            });
    }
}
