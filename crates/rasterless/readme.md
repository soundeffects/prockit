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
- wgpu raytracers: https://github.com/albedo-engine/albedo/, https://github.com/oxabz/wgpu-raymarcher/
- bevy_render internals: https://github.com/bevyengine/bevy/tree/main/crates/bevy_render
- Summary of bevy's rendering plugins: https://hackmd.io/@nth/rendering_summary
- Data between CPU and GPU: https://github.com/schell/crabslab
- GPU driven renderers, wgpu: https://github.com/schell/renderling, https://github.com/pudnax/voidin
- compute shaders in bevy: https://odysee.com/@LogicProjects:4/compute-shaders-in-bevy:a
- WGSL syntax guides: https://www.w3.org/TR/WGSL/, https://google.github.io/tour-of-wgsl/
- GPU-driven rendering overview: https://vkguide.dev/docs/gpudriven/gpu_driven_engines/

## Notes
Currently working on allocating a voxel storage buffer. Need to make sure that a) a large (several gigabytes) buffer
can be allocated without errors, b) that buffer can be written to from the CPU, and c) make sure it can be read and
written to on the GPU correctly.

The current implementation has defined some new limits for the GPU device, otherwise buffers larger than 256 Mb can't
be allocated. It has the flags COPY_DST and STORAGE on the buffer, hopefully that will give the CPU and GPU access
patterns we're expecting. However there is an error where the Rust-side buffer binding defines as 52-byte sized object,
and the WGSL shader code expects a 64-byte sized object.

Exact error message:
Buffer is bound with size 52 where the shader expects 64 in group[0] compact index 0

