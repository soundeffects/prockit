use bevy::{
    ecs::{entity_disabling::Disabled, query::QueryData},
    prelude::*,
    utils::{TypeIdMap, TypeIdMapExt},
};
use std::{any::Any, marker::PhantomData};

pub trait PoolTraits: Default + Send + Sync + 'static {}
pub trait Spawner<Q: QueryData>: Fn(Q::Item<'_, '_>) + Send + Sync + 'static {}

#[derive(Default, Resource)]
pub struct Pool<PoolId: PoolTraits> {
    memory_limit: usize,
    queues: TypeIdMap<Vec<Box<dyn Any + Send + Sync>>>,
    defaults: TypeIdMap<(Box<dyn Any + Send + Sync>, u16)>,
    pooling: Vec<Entity>,
    _id: PhantomData<PoolId>,
}

impl<PoolId: PoolTraits> Pool<PoolId> {
    pub fn new(memory_limit: usize) -> Self {
        Self {
            memory_limit,
            ..default()
        }
    }

    pub fn register<Q: QueryData + 'static>(&mut self, default: impl Spawner<Q>) {
        let boxed: Box<dyn Any + Send + Sync> = Box::new(default);
        self.queues.insert_type::<Q>(Vec::new());
        self.defaults.insert_type::<Q>((boxed, 0));
    }

    pub fn spawn<Q: QueryData + 'static>(&mut self, callback: impl Spawner<Q>) {
        if let Some(queue) = self.queues.get_type_mut::<Q>() {
            let boxed: Box<dyn Any + Send + Sync> = Box::new(callback);
            queue.push(boxed);
        }
    }

    pub fn spawn_default<Q: QueryData + 'static>(&mut self) {
        if let Some((_, spawns)) = self.defaults.get_type_mut::<Q>() {
            *spawns += 1;
        }
    }

    pub fn pool(&mut self, entity: Entity) {
        self.pooling.push(entity);
    }

    fn update<Q: QueryData + 'static>(
        mut commands: Commands,
        mut pool: ResMut<Pool<PoolId>>,
        mut query: Query<(Entity, Q), With<Disabled>>,
    ) {
        let mut query_iter = query.iter_mut().peekable();
        if let Some(queue) = pool.queues.get_type_mut::<Q>() {
            while !queue.is_empty() && query_iter.peek().is_some() {
                let callback = queue
                    .pop()
                    .unwrap()
                    .downcast::<Box<dyn Fn(Q::Item<'_, '_>)>>()
                    .unwrap();
                let (entity, pooled) = query_iter.next().unwrap();
                callback(pooled);
                commands.entity(entity).remove::<Disabled>();
            }
        }

        if let Some((spawner, spawns)) = pool.defaults.get_type_mut::<Q>() {
            if *spawns > 0 {
                let spawner = spawner.downcast::<Box<dyn Fn(Q::Item<'_, '_>)>>().unwrap();
                while query_iter.peek().is_some() && *spawns > 0 {
                    let (entity, pooled) = query_iter.next().unwrap();
                    spawner(pooled);
                    *spawns -= 1;
                    commands.entity(entity).remove::<Disabled>();
                }
            }
        }

        let size = pool.pooling.len();
        for entity in pool.pooling.drain(0..size) {
            commands.entity(entity).insert(Disabled);
        }
    }

    fn recalibrate(mut pool: ResMut<Pool<PoolId>>) {
        // TODO: Need size on Q
        // TODO: Should collect counts of allocations of each type in history window into a
        // histogram. Compare relative to histogram of current reserves, perform some
        // allocations or deallocations (if no allocations can be made) to make reserves
        // relatively line up.
    }
}

#[derive(Default)]
pub struct PoolPlugin<PoolId: PoolTraits> {
    memory_limit: usize,
    pool_reg: Vec<Box<dyn Fn(&mut Pool<PoolId>) + Send + Sync>>,
    app_reg: Vec<Box<dyn Fn(&mut App) + Send + Sync>>,
}

impl<PoolId: PoolTraits> PoolPlugin<PoolId> {
    pub fn new(memory_limit: usize) -> Self {
        Self {
            memory_limit,
            ..default()
        }
    }

    pub fn with<Q: QueryData + 'static>(mut self, default: impl Spawner<Q>) -> Self {
        self.app_reg.push(Box::new(|app| {
            app.add_systems(Update, Pool::<PoolId>::update::<Q>);
        }));
        self.pool_reg.push(Box::new(|pool| {
            pool.register::<Q>(default);
        }));
        self
    }
}

impl<PoolId: PoolTraits> Plugin for PoolPlugin<PoolId> {
    fn build(&self, app: &mut App) {
        let mut pool = Pool::<PoolId>::default();
        for pool_call in &self.pool_reg {
            pool_call(&mut pool);
        }
        app.insert_resource(pool)
            .add_systems(Update, Pool::<PoolId>::recalibrate);
        for app_call in &self.app_reg {
            app_call(app);
        }
    }
}
