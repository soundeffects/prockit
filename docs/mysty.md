# Mysty: Requirements
The Mysty game project is meant to achieve an ideal that Minecraft modding has inspired in me. Below, I lay out my best conception of the requirements of that ideal game.

## Continuations
- Easy but conceptually deep building mechanics
- Simple and straightforward interactions with the world from which complex and emergent interactions follow
- Simplistic 3D art style that piques the imagination
- Interesting fantastical but grounded terrain generation
- Crafting pipelines, automation, resource gathering and item unlocks as progression and gameplay loop
- Declarative modular content in the form of datapacks and resourcepacks
- Platform for content sharing
- Focus on "incarnation" in the game world: there are some RPG-like adventuring goals, but mostly intrinsic and social motivations for playing
- Basic graphics pipeline that runs on all systems (OpenGL, triangles)
- Big focus on accessibility and universality
- Self-hostable multiplayer

## Refinements
- Systems programming language for the important bits
- LOD system and simplistic shaders (minimal shadowcasting, more focus on flood lighting, ambient occlusion, and textures) for graphical improvements
- Research minigame to find and chart the characteristics of materials, items, and crafting methods
- Physics system for blocks, fluids, and cloths
- Difficulty setting is the presence of survival mechanics that must be managed
- Players know when they are walking into danger, or when danger is coming to them.
- Intuitive way to keep track of knowledge, minimal "meta-gaming" feel (looking at JEI as an example of "meta-gaming")
- Variety in the environment and with building, with out micro-managing inventory and material types (texture variants, etc.)
- Clean UI for what menus are required
- Smooth and satisfying movement and combat--a natural feel to "incarnation"
- Make the world feel alive by making a lot of small, inconsequential details for your goals but which are interesting to observe
- Hobbies, side projects and minigames within the game itself, deep enough to devote your time to, are often times more important than having an overarching goal
- Room for several genres, settings, and "feels" within the core mechanics

## Explorations
- Allow more natural placement of terrain and buildings, using diagonal and smooth surfaces for walls and floors.
- Generated materials, plants and creatures, unique for every world. Generate models, textures, animations, behaviors, and ecosystem niches for all.
- Generated civilization, from villagers which interact and socialize (in a way that is as simplistic as possible while still being satisfying for the player) to the towns that they form to the kingdoms and politics which those form.
- Exploring various AI paradigms for the above generations
- Simulating larger events that effect the world, and then propogating them to the local area around the player
- Dialogue with NPC's
- Principle of progressive refinement for close-up details
- Immersive and "VR-first" design, with minimal menus involved in crafting and interacting, in-depth in-world crafting
- "Smooth" voxels or alternative representations of space
- A lengthy, cooperative goal for many players to work towards within the game, and which becomes a community achievement upon finishing
- Virtual economies, for both NPC's and players
- Loosely-connected worlds, where you can seek out solitude or seek out a small amount of company all within the game
- "Isekai experience", wher you progress and build relationships in the world, and eventually come to a position of authority or management in your own area of interest
- An AI "gamemaster" which wants to tell a story using the world before it, and tries to set up events in a narratively satisfying way

## Inspirations
- Minecraft (modding specifically)
- Rimworld (along with mods)
- Baldur's Gate 3
- Enshrouded (it has a similar premise but several pitfalls that should be avoided)
- Veloren (feature-poor but still an example of a community led effort for a voxel game)
- Minetest (an open source minecraft clone)
- Terraria (excellent item-based and world-discover-based progression)
- No Man's Sky (hidef voxel-based terrain)
- Vintage Story (In-world crafting, complex survival mechanics)
- Tiny Glade (Node-based procedurally buildings and walls)

## Technical Foundations
As long as the foundations set up the project to achieve the goals above within the span of a few years of work, we will prioritize simple and proven technologies.
- Engine in Rust: reduces bugs and testing needed for confidence, which speeds up long projects
- Terrain consists of heightmap with sparse "Transvoxel" areas for caves/overhangs: for large worlds we need to reduce unnecessary volumetric data
- Buildings and structures are node-dictated small-mesh-instantiation procedural shapes
- Low-poly artstyle
