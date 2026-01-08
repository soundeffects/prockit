# Plan: Provides
one of the names in a `Names` struct.
The `Provides` struct should be constructed by providing it functions (stored as `dyn Any` and
later downcasted back into the appropriate function) and their `Signature`.

`Signature` structs should have up zero to four argument types and a return type, all stored as
`NamedType` structs. They should be constructed using a trait `SignatureFrom<T>` implemented for
the `Signature` type for each possible function type (zero to four arguments all are different
function types). Finally, `Signature` structs should store whether they require a reference or
a reference to the type that creates the `Provides` as the first argument (`&self` or
`&mut self`) as an `enum`.

*Revised: Self references will instead be auto-detected in the first argument position by
wrapping layers above the `Provides` struct.*

`NamedType` structs should store a `TypeId` and a `Names` struct.

`Names` structs should store a list of names as strings.

The `Provides` struct can then be queried for functions it stores using a `SignatureQuery`
struct. Any `SignatureQuery` which matches a `Signature` should return the underlying function
(downcasted using the function type which the `SignatureQuery` was constructed from).

A `SignatureQuery` struct should have zero to four argument types and a return type, all stored
as `NamedTypeQuery` structs. They should be constructed similarly to the `Signature` struct, by
using a trait `SignatureQueryFrom<T>` implemented for all possible function types. Finally, the
`SignatureQuery` should also store whether they expect a reference (`&self` or `&mut self`) as
an `enum`. A `SignatureQuery` should match a `Signature` if all `NamedTypeQuery` members match
the `Signature` `NamedType` members, and if the reference `enum` is equal.

`NamedTypeQuery` structs should store a `TypeId` and a `NameQuery` struct. A `NamedTypeQuery`
should match a `NamedType` if the `NameQuery` member matches the `NamedType` `Names` member,
and if the `TypeId`s are equal.

`NameQuery` structs should store a regular expression (using the crate `regex`) used to
search for matches in a `Names` struct.
