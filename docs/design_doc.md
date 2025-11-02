## Graphics Developer Goals

My obsessions:
- Photorealistic rendering
    Bevy: some maintainers care. Slowly getting there.
- Rendering data that is more abstract and expressive than triangle meshes
    Bevy: very few maintainers care. Mostly my own work.
- Deep simulation of game worlds
    Bevy: fairly aligned, but not explicitly aligned.
- Procedurally generated content, including AI generated content
    Bevy: somewhat aligned, but few people care. Not many maintainers, but some crates.
- VR
    Bevy: some maintainers care. Slowly getting there.
- Optimizing experience and delivery to as many platforms as possible
    Bevy: strongly aligned.
- DX and UX
    Bevy: strongly aligned.
- "Socially intelligent" networking
    Bevy: early stages, but may evolve to be important to the community.
- User generated content
    Bevy: fairly aligned. On the backburner, but maintainers are working towards the groundwork.

I don't really care where I work, as long as its not too draining and compensates well. Ideal world, I would work
in open source and towards all the goals stated above. Realistically, I work for a company on something I only care
moderately for, and have to get myself to split my time fairly so that I meet expectations at work. I'm fine with
towing the line on expectations at work if it means I can work towards the above goals better.

To complete my masters project, at a minimum I must:
- Create a perception model that maps a camera view over time into a spatial/per-object attention field
- Visualize the attention field
- Demonstrate that the perception model matches user descriptions of what they were most attentive of.

Types of rendered objects:
- Static: These do not deform but may translate/rotate. Can be modelled using voxels.
- Skeletal: These can be represented by DAGs of vertices which may move and deform. Should be modeled using "skeleton" mesh generation.
- Fluid: These are particle simulations. Should be modelled using adaptive particle clouds.
- Cloth: These are spring-jointed meshes that usually are physically simulated. Should be modeled using simple meshes.
- Particles: These are simple, individual particles. Should be modelled using particle emitters and basic billboards.
- Screens: For UI windows and pictures. Can be modeled as quads in space.
- Portals: Two planes/2D shapes which are connected. Can be modeled using 2D meshes/shapes and another camera render pass
- Contraptions: Combinations of the above, which interact in special ways. Should be simply modelled as a collection of constituent parts.
- Bespoke: Triangle meshes which fall into none of the above categories, and do not have any notable interactions other than rendering and
    basic animation.

From the above, my current priorities should be:
1. Do the minimum to complete the masters project
2. Contribute to and advertise bevy_voxel_world and bevy_skeletons to demonstrate the usefulness of data abstraction
3. Build a "game kit" to promote use of data abstraction and pretty photorealistic graphics
4. Build a renderer for this game kit that is specifically tailored to it and photorealism