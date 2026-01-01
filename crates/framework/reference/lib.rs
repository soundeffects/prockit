mod mod_entry;
mod names;
mod provides;
mod providing;

use bevy::prelude::*;
pub use mod_entry::{Entry, NamedType};
pub use names::{CONVENTIONS, Convention, Names};
pub use provides::{Needs, Provides};
pub use providing::Providing;

// TODO: Saveable, Renderable, and Simulatable Nodes
// TODO: Hierarchical provider collection
// TODO: Children collection for compression
// TODO: Combo node for multiple procedural nodes in one
// TODO: Transform to bounds on ProceduralNode

pub trait ProceduralNode {
    fn view(&self, bounds: &Bounds, distance: f32, time: f32) -> ViewAction;

    fn enhance(&self, bounds: &Bounds, provider: &Provides, node_commands: NodeCommands);

    fn init() -> Self
    where
        Self: Sized;

    fn provides(&self) -> Provides;

    fn generate(&mut self, bounds: &Bounds, provider: &Provides) -> GenerateAction;
}

pub struct NodeCommands<'a, 'b, 'c> {
    commands: &'c mut Commands<'a, 'b>,
    provider: &'c Provides,
}

impl NodeCommands<'_, '_, '_> {
    pub fn add_child<T: ProceduralNode + Send + Sync + 'static>(&mut self, bounds: &Bounds) {
        let mut node = T::init();
        node.generate(bounds, self.provider);
        self.commands.spawn(ProceduralNodeWrapper {
            node: Box::new(node),
            bounds: bounds.clone(),
        });
    }
}

pub enum ViewAction {
    Enhance,
    Compress,
    None,
}

pub enum GenerateAction {
    RegenerateChildren,
    None,
}

pub struct ProckitFrameworkPlugin;

impl Plugin for ProckitFrameworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, resample_nodes);
    }
}

#[derive(Clone)]
pub struct Bounds;

#[derive(Component)]
struct ProceduralNodeWrapper {
    node: Box<dyn ProceduralNode + Send + Sync>,
    bounds: Bounds,
}

impl ProceduralNodeWrapper {
    fn enhance(&self, provider: Provides, commands: &mut Commands) {
        self.node.enhance(
            &self.bounds,
            &provider,
            NodeCommands {
                commands,
                provider: &provider,
            },
        );
    }

    fn compress(&mut self, _children: Vec<ChildOf>) {
        self.node.generate(&self.bounds, &Provides::default());
    }
}

fn resample_nodes(
    mut commands: Commands,
    mut nodes: Query<(&mut ProceduralNodeWrapper, Option<&ChildOf>)>,
) {
    for (mut wrapper, parent) in &mut nodes {
        let provider = if let Some(ChildOf(_entity)) = parent {
            // nodes.get(*entity).unwrap().0.node.provides()
            Provides::default()
        } else {
            Provides::default()
        };
        match wrapper.node.view(&wrapper.bounds, 0., 0.) {
            ViewAction::Enhance => {
                wrapper.enhance(provider, &mut commands);
            }
            ViewAction::Compress => {
                wrapper.compress(vec![]);
            }
            ViewAction::None => (),
        }
    }
}
