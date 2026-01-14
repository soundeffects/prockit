# Lifetime-Based Method Currying Implementation

## Overview

This document describes the implementation of self-referential method currying for the prockit framework's `Provides` and `Provider` types, enabling methods with `&self` arguments to be registered and queried as if they had no `&self` argument.

## Key Changes

### 1. Lifetime Parameters

Added lifetime parameter `'a` to both `Provides<'a>` and `Provider<'a>`:

```rust
pub struct Provides<'a> {
    entries: Vec<FunctionEntry<'a>>,
}

pub struct Provider<'a> {
    provides_chain: Vec<Provides<'a>>,
}
```

This allows closures to borrow data for a specific lifetime instead of requiring `'static`.

### 2. Raw Pointer Type Erasure

Bypassed `std::any::Any`'s `'static` requirement by implementing custom type erasure:

```rust
struct FunctionEntry<'a> {
    signature: Signature,
    type_key: TypeId,           // For safe downcasting verification
    function_ptr: *const (),    // Type-erased function pointer
    drop_fn: fn(*const ()),     // Proper cleanup
    _marker: PhantomData<&'a ()>,
}
```

Functions are stored as raw pointers with a `TypeId` tuple key (e.g., `TypeId::of::<(R, A1, A2)>()`) for type safety. Unsafe casting is validated by matching the type key before retrieval.

### 3. Method Currying API

Added `add_method_*` methods (0-4 arguments) to `Provides` that curry the `&self` receiver:

```rust
impl<'a> Provides<'a> {
    pub fn add_method_3<S, R, A1, A2, A3>(
        &mut self,
        receiver: &'a S,
        method: fn(&S, A1, A2, A3) -> R,
        r_names: Names,
        a1_names: Names,
        a2_names: Names,
        a3_names: Names,
    ) where S: Sync { ... }
}
```

This allows methods to be registered using `self.add_method_3(self, Self::method, ...)` and queried as 3-argument functions.

### 4. ProceduralNode Trait Update

Changed from:
```rust
fn provides(&self) -> Provides<'static>
```

To:
```rust
fn register_provides<'a>(&'a self, provides: &mut Provides<'a>)
```

This allows nodes to register methods that borrow from `&self` during their lifetime.

### 5. iter_many Integration

Restructured provider collection to use Bevy's `iter_many` for simultaneous multi-entity borrowing:

```rust
fn build_provider_from_refs<'a>(
    ancestor_refs: &'a [Ref<'a, dyn ProceduralNode>],
) -> Provider<'a>
```

This holds all ancestor node references simultaneously, making the provider's lifetime valid throughout subdivision.

## Usage Example

```rust
impl ProceduralNode for Chunk {
    fn register_provides<'a>(&'a self, provides: &mut Provides<'a>) {
        provides.add_method_3(
            self,
            Self::opaque,
            Names::from("opaque"),
            Names::from("x"),
            Names::from("y"),
            Names::from("z"),
        );
    }

    fn subdivide(
        &self,
        transform: &GlobalTransform,
        provider: &Provider<'_>,
        mut child_commands: ChildCommands,
    ) {
        // Query parent's opaque method as a 3-arg function
        let opaque = provider.query_3::<bool, usize, usize, usize>(
            NameQuery::exact("opaque"),
            NameQuery::exact("x"),
            NameQuery::exact("y"),
            NameQuery::exact("z"),
        ).unwrap();

        // Call without &self - it's curried in
        if opaque(x, y, z) { ... }
    }
}
```

## Safety

- Raw pointer usage is encapsulated in `FunctionEntry`
- Type safety enforced via `TypeId` matching before unsafe casts
- Proper cleanup via stored `drop_fn` function pointer
- Lifetimes ensure borrowed data outlives the closures

## Testing

- 21 unit tests verify core functionality
- 25 doc tests ensure API examples work
- Tests specifically verify lifetime safety and method currying behavior
