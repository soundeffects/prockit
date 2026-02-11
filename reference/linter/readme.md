# Prockit Linter
There are some general tips that improve the discoverability of the functions that users
and developers might provide each other through the prockit framework. These will be
directly linked to the development experience using a custom linter, which has yet to be
implemented.

## Planned Lints
- Duplicate names for a single `NamedType`
- `NamedType` has names associated to types that do not follow prockit conventions
- More than eight names for a single `NamedType`
- `Name`s with more than sixteen characters
- More than four arguments for a single `Entry`
- `Entry` has two or more arguments with duplicate names
- `Provides` has two or more `Entry` types with duplicate return names
- `Name`, `Names`, or any dependent type has been instantiated with dynamic values instead
of literals (which isn't recommended because it can break above lints)

## Resources
- [Write your own rust linter](https://blog.guillaume-gomez.fr/articles/2024-01-18+Writing+your+own+Rust+linter)
