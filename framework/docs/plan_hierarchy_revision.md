# Plan: Hierarchy Revision

## Premise, As Logical Deduction
1. In order to generate an infinite world, it must be modularized

2. In order to generate an infinite number of modules, they need to be prioritized
    - We prioritize by some metric of "noticeability" to the viewers of the world
    - When "noticeability" changes, we may drop some unnoticeable modules and work on new,
    more noticeable modules

3. In order to generate an infinite number of modules, there needs to be a stopping point
    - This includes the limits of the computer (compute/memory)
    - This includes details the viewers will not notice

4. Generating a module from nothing is an unconstrained creative process
    - Note that the generation process should be async, in that it does not need to complete on
    short order

5. However, there are properties of the world which we desire to be consistent, usually at a
specific scale, and not unconstrained

6. Modules are limited to influencing a local region, with a specific property size, otherwise
the responsibilities of a module becomes infinite again

7. With modules that all match the same scale, it becomes difficult to maintain consistency for
properties of higher or lower scales

8. Thus, we need to organize a process by which modules of higher scales or lower scales can be
generated from the existing set of modules, and enforce consistency for some properties

9. Thus, we use a hierarchy of modules

10. Modules must state the properties they want to govern for consistency's sake, and the
program managing the hierarchy should enforce that consistency
    - They will ideally also specify how much those properties can err when other modules are
    generated in a consistent manner--we may wish to have some small amount of variability

11. Note that when modules are consistent at varying scales, we may sample the property
consistently at whatever scale we need
    - If we wish to check a property in the world, we should use a "detail scale"

12. When updating a property at any scale, we need to update modules higher and lower in the
hierarchy such that they report the property consistently with the update
    - This update process should be async, in that it does not need to complete in short order,
    but it should respect ordering of property updates

13. Modules will cover a local region, and consistent properties may vary over the local region
that a module governs

14. Thus, modules will need to advertise not just constant properties, but functions which take a sampling position and return the value and error limit of the property that must be consistent

15. Modules may exist in many different spaces, and possibly multiple spaces at once
    - Examples include three-dimensional space, two-dimensional space, etc.

16. Thus modules will also need to advertise the space upon which a property exists, and spaces
must define what type a sampling position will be

17. When the program decides on generating a new module, it should provide the module with the
region of space(s) it governs, a detail size for each space, and the ability to query the
existing properties of that region of space

18. Each space can have a variety of placement types. For 3D space, it might be the placement
types of "volume subdivide", "node subdivide", "volume scatter", and "surface scatter". When
queried for a placement type, a procedural node will describe all the possible placements of
children for that type, and will describe the local region and detail scale for each.

19. Procedural node types will also describe a method to accept a placement. They will be
provided with the placement type, the local region, detail scale, and the properties defined
at each placement. The procedural node will be generated if the placement is accepted. The
system will try each procedural node type that has been registered in a random order until one
or none are accepted at the placement.

## Implementation Notes
- `ProceduralNode` should be a trait, with methods for all of the characteristics described
above
- The `Provider` class should be used to pass on properties as functions. It should also pass on
a local region, which is a type defined by the space, and a detail scale, which is a float.
- The `Space` trait should require a position type, should require a method for returning a
`Vec` of placement types, should require a local region type, and should describe a method for
checking noticeability of procedural nodes.
