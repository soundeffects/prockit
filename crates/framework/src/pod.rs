use super::{ProceduralNode, Space};
use bevy::prelude::*;
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

#[derive(Component)]
pub struct Pod<S: Space, T: ProceduralNode<S>> {
    data: Arc<RwLock<T>>,
    _space: PhantomData<S>,
}
