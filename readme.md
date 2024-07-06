# Rasterless Renderer
An experiment to develop a realtime pathtracing rendering engine which uses voxel grids, implicit surfaces,
and other scene data formats rather than triangle meshes.

This is still WIP, and there's nothing to show yet. Check back later!

## References
The following are mostly my own notes, but they indicate directions I'm looking to take the code, and they
may be informative to a visitor!

- For voxels on disk: https://github.com/cberner/redb
- For the 'zfp' fast spatial data compression algorithm: https://zfp.readthedocs.io/en/release1.0.1/algorithm.html#algorithm
- For implementation details on OpenVDB: https://www.museth.org/Ken/Publications_files/Museth_TOG13.pdf
- For an example of a custom bevy rendering pipeline: https://github.com/Lommix/bevy_pipeline_example/blob/master/src/render.rs
- For a presentation on GPU driven rendering pipelines: https://advances.realtimerendering.com/s2015/aaltonenhaar_siggraph2015_combined_final_footer_220dpi.pdf
- Collection of graphics articles: https://www.jendrikillner.com/#about
- wgpu raytracers: https://github.com/jgrazian/wgpu-raytracer, https://github.com/albedo-engine/albedo/tree/main, https://github.com/oxabz/wgpu-raymarcher/tree/main
- bevy_render internals: https://github.com/bevyengine/bevy/tree/main/crates/bevy_render
- Summary of bevy's rendering plugins: https://hackmd.io/@nth/rendering_summary
- Data between CPU and GPU: https://github.com/schell/crabslab
- GPU driven renderers, wgpu: https://github.com/schell/renderling, https://github.com/pudnax/voidin
- compute shaders in bevy: https://odysee.com/@LogicProjects:4/compute-shaders-in-bevy:a
