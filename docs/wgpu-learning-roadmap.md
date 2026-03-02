# WGPU Learning Roadmap

## Core Docs

- `wgpu` API reference: https://docs.rs/wgpu/latest/wgpu/
- Learn WGPU tutorial: https://sotrh.github.io/learn-wgpu/
- WebGPU spec: https://www.w3.org/TR/webgpu/
- WGSL shader spec: https://www.w3.org/TR/WGSL/
- `winit` API reference: https://docs.rs/winit/latest/winit/
- Naga (shader translation): https://github.com/gfx-rs/naga

## 4-Week Practical Plan

### Week 1: Foundations

- Read Learn WGPU intro + window/surface setup.
- Build: clear screen with your current app.
- Goal: understand `Instance`, `Surface`, `Adapter`, `Device`, `Queue`, `SurfaceConfiguration`.

### Week 2: Pipeline + Geometry

- Learn render pipeline, shaders, vertex/index buffers.
- Build: render one triangle, then a quad.
- Goal: understand GPU pipeline stages and draw calls.

### Week 3: Uniforms + Camera

- Learn uniform buffers + bind groups.
- Build: move/zoom camera over a 2D grid.
- Goal: pass dynamic data from CPU to GPU each frame.

### Week 4: Textures + Optimization

- Learn textures/samplers + depth basics.
- Build: simple textured cells for Game of Life.
- Goal: render efficiently and handle resize/surface errors cleanly.

## Suggested Next Step in This Project

- Move `WgpuState` from `window.rs` into `renderer.rs`.
- Keep `window.rs` focused on events/lifecycle only.
