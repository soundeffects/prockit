# Plan: Procedural Node Hierarchy
The `ProceduralNode` trait defines several methods. Dynamic `ProceduralNode` objects are
managed by a Bevy plugin after the concrete type implementing `ProceduralNode` has been
registered at app startup. The plugin manages the `ProceduralNode`s in the following ways,
using the user-provided implementations of the `ProceduralNode` methods.

## Dependencies
We require `bevy-trait-query` such that we can use Bevy queries to find all `ProceduralNode`
objects as dynamic objects.

## Provider Collection
*Corresponding trait function: `fn provides(&self) -> Provides`.*

`ProceduralNode` requires an implementation of `ProceduralNode::provides`, which returns a
`Provides` struct. Many of the other `ProceduralNode` methods require a `Provider` input.
This is constructed ad hoc when these methods are called by collecting all the `Provides`
of all ancestors (in terms of Bevy's parent/child hierarchy) of the `ProceduralNode` calling
the method. The `Provides` are added to the `Provider` in order of hierachy, meaning the
direct parent of the original `ProceduralNode` is placed first and all greater ancestors are
placed in ascending order, until the root node is last.

`Provider` structs are a collection of `Provides` structs, as defined above. `Provider`s allow
querying functions with the same interface as `Provides`, but do not allow adding functions.
When querying functions from a `Provider`, it will call the corresponding query on each
`Provides` it holds in the order they were placed when the `Provider` was constructed. This
means that any overlapping function signatures from two different `Provides` will return the
function definition of the closest ancestor when queried.

## Updating
*Corresponding trait function: `fn update(&mut self, provider: &Provider)`.*

For all `ProceduralNode` objects which have changed (as according to Bevy's `Changed` query
filter), the `ProceduralNode::update` function should be called on all of their children, with
the appropriate `Provider` passed in (as described in the "Provider Collection" section).

## Distance
*Corresponding trait function:
`fn min_square_distance(&self, transform: GlobalTransform, viewer: Vec3) -> f64`.*

`ProceduralNode` objects are structured in a level-of-detail hierarchy, which generates the
greatest amount of detail close to the viewer. When the plugin is determining whether to
increase or decrease the level-of-detail for a `ProceduralNode`, it will iterate over the
transforms of each `Viewer` (which is a marker Bevy `Component`), find the viewer's minimum
square distance using the `ProceduralNode::min_square_distance` function, and select the
minimum of all viewers. It will compare that number to the increase/decrease threshold set in a
`ProckitFrameworkConfig` struct, which is a Bevy `Resource`.

## Emptiness
*Corresponding trait function: `fn is_empty(&self) -> bool`.*

The `is_empty` function is the way a `ProceduralNode` can signal that it should not be
subdivided, even if it is otherwise eligible to be subdivided.

## Increasing Detail
*Corresponding trait function:
`fn subdivide(&self, provider: &Provider, child_commands: ChildCommands)`.*

For all `ProceduralNode` objects which have no children, and which have a distance that passes
the threshold for increasing the level-of-detail, and which are not empty (see the "Empty"
section), they will be subdivided by calling the `ProceduralNode::subdivide` method. This method
generates new, higher detail `ProceduralNode`s as Bevy children. This method gets access to a
`Provider`, as described in the "Provider Collection" section. It also gets access to a
`ChildCommands` struct, which wraps the Bevy `Commands` struct and exposes only one method:
`add_child`, which will add a Bevy `Bundle` provided to it as a child to the `ProceduralNode`
that has been given the `ChildCommands`.

## Decreasing Detail
For all `ProceduralNode` objects which are direct parents of any `ProceduralNode` objects with
no children, and which have a distance that passes the threshold for decreasing the
level-of-detail, all of their children will be despawned.

## Optimization Notes
It may be beneficial to implement `Provider` such that it can be incrementally constructed while
traversing the `ProceduralNode` hierarchy. For example, it may take the `Provider` constructed
by a parent and append the `Provides` for each child as it traverses into that child. This would
allow a single pass of the `ProceduralNode` hierarchy to handle the steps described in "Provider
Collection", "Updating", "Increasing Detail", and "Decreasing Detail" sections.
