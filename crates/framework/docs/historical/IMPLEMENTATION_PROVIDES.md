# Provides Implementation Documentation

## Overview

The `Provides` struct has been successfully implemented according to the requirements in `plan_provides.md`. This implementation provides a flexible system for storing and querying functions with named parameters and return types using regex-based pattern matching.

## File Location

`src/provides.rs` - New file containing the complete implementation

## Components Implemented

### 1. Core Structures

#### `SelfReference` Enum
```rust
pub enum SelfReference {
    None,  // Function doesn't require self reference
    Ref,   // Function requires &self
    Mut,   // Function requires &mut self
}
```
Tracks whether a function requires a reference to the providing type.

#### `Names` Struct
- Stores multiple name aliases for types and parameters as strings
- Supports iteration and lookup
- Implements `From` traits for convenient construction from `&str`, `String`, and arrays

#### `NamedType` Struct
- Combines a `TypeId` with a `Names` struct
- Represents a type along with its possible names
- Used for both function parameters and return types

#### `Signature` Struct
- Stores complete function signature information:
  - `self_reference`: Whether the function needs `&self` or `&mut self`
  - `return_type`: NamedType for the return value
  - `arg_types`: Vec of NamedType for parameters (supports 0-4 arguments)

### 2. Signature Construction

#### `SignatureFrom<F>` Trait
- Implemented for function types with 0-4 arguments
- Takes a tuple of `Names` and constructs a `Signature`
- Implementations for:
  - `(Names,)` for `fn() -> R`
  - `(Names, Names)` for `fn(A1) -> R`
  - `(Names, Names, Names)` for `fn(A1, A2) -> R`
  - `(Names, Names, Names, Names)` for `fn(A1, A2, A3) -> R`
  - `(Names, Names, Names, Names, Names)` for `fn(A1, A2, A3, A4) -> R`

### 3. Query Structures

#### `NameQuery` Struct
- Uses regex patterns to match against `Names`
- Provides convenient constructors:
  - `exact(name)`: Match exact name
  - `from_pattern(regex)`: Custom regex pattern
- Implements `From<&str>`, `From<String>`, and `From<Regex>`

#### `NamedTypeQuery` Struct
- Combines `TypeId` with `NameQuery`
- Matches against `NamedType` by checking both type and name

#### `SignatureQuery` Struct
- Represents a query for finding functions
- Matches against `Signature` by comparing:
  - Self reference requirement
  - Return type (both TypeId and names)
  - Argument types (both TypeId and names, in order)

### 4. Query Construction

#### `SignatureQueryFrom<F>` Trait
- Implemented for function types with 0-4 arguments
- Takes a tuple of `NameQuery` and constructs a `SignatureQuery`
- Parallel implementations to `SignatureFrom` for all function arities

### 5. Main Provides Struct

#### `Provides`
- Stores functions as `Box<dyn Any + Send + Sync>` with their signatures
- **Methods:**
  - `new()`: Create empty Provides
  - `add<F, S>()`: Add a function with its signature
    - `F`: Function type (must be `'static + Send + Sync`)
    - `S`: Signature data (tuple of Names implementing `SignatureFrom<F>`)
    - `self_reference`: Whether function needs self reference
  - `query<F, Q>()`: Query for a function
    - `F`: Function type to retrieve
    - `Q`: Query data (tuple of NameQuery implementing `SignatureQueryFrom<F>`)
    - Returns `Option<&F>` - reference to function if found

## Usage Example

```rust
use prockit_framework::{Provides, Names, NameQuery, SelfReference};

let mut provides = Provides::new();

// Add a function
fn add(x: i32, y: i32) -> i32 {
    x + y
}

provides.add(
    add as fn(i32, i32) -> i32,
    (Names::from("sum"), Names::from("x"), Names::from("y")),
    SelfReference::None,
);

// Query for the function
let result = provides.query::<fn(i32, i32) -> i32, _>(
    (NameQuery::from("sum"), NameQuery::from("x"), NameQuery::from("y")),
    SelfReference::None,
);

if let Some(func) = result {
    assert_eq!(func(2, 3), 5);
}
```

## Key Features

1. **Type Safety**: All functions are stored with full type information
2. **Named Parameters**: Parameters can have multiple aliases
3. **Regex Matching**: Flexible name matching using regex patterns
4. **Self Reference Tracking**: Distinguishes between functions, methods with `&self`, and methods with `&mut self`
5. **Thread Safe**: Functions must implement `Send + Sync` for use in Bevy systems

## Integration with Existing Code

The implementation integrates with the existing `src/lib.rs`:
- Exported all public types from the `provides` module
- The old `Provides` struct in `lib.rs` was removed (lines 144-160)
- `Entry` struct remains as a placeholder in `lib.rs` (may be used differently)
- Compatible with Bevy's `Resource` trait (Provides implements Send + Sync)

## Test Coverage

Comprehensive test suite includes:
- Names creation and lookup
- NamedType construction
- Exact and regex name queries
- NamedTypeQuery matching
- Signature construction for various arities
- SignatureQuery matching
- Provides add and query operations
- Self reference matching
- Multiple name aliases

All 12 tests pass successfully.

## Conformance to Plan

The implementation fully conforms to `docs/plan_provides.md`:
- ✅ Functions stored as `dyn Any` and downcasted
- ✅ Signatures with 0-4 arguments and return type
- ✅ SignatureFrom trait for construction
- ✅ Self reference tracking (None/Ref/Mut)
- ✅ NamedType with TypeId and Names
- ✅ Names with list of strings
- ✅ SignatureQuery for querying
- ✅ NamedTypeQuery matches NamedType
- ✅ NameQuery with regex matching
- ✅ All matching logic implemented

## Future Enhancements

Potential improvements not in the original plan:
- Support for more than 4 arguments
- Macro for easier function registration
- Better error messages for failed queries
- Query result ranking (when multiple functions match)
