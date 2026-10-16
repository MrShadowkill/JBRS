# Jailbreak RS — AI Engineering Guidelines
## Rust / Legacy OpenGL / Android / Physics Sandbox

**Status:** Mandatory engineering specification  
**Priority:** Hardware compatibility → correctness → performance → gameplay → visuals.

All coding agents working on this repository MUST follow these rules.

---

# 1. TARGET HARDWARE — STRICT

## Primary PC target

- CPU: Intel Core i7-620M, 2 cores / 4 threads.
- GPU: Intel HD Graphics Ironlake (1st generation).
- RAM: 8 GB DDR3-1066.
- OS: Linux Mint / Ubuntu / Debian.
- Driver: Mesa legacy Intel graphics driver.
- Graphics API: OpenGL 2.1 compatibility profile.
- Shader language: GLSL 1.20.

NEVER require:
- OpenGL 3.0+.
- Vulkan.
- Compute shaders.
- Geometry shaders.
- Tessellation.
- Shader Storage Buffer Objects.
- Uniform Buffer Objects.
- Direct State Access.
- Core-profile-only functionality.

Optional OpenGL extensions MUST be checked at runtime.

Never assume extension availability.

## Performance targets

| Setting | Resolution | Target |
|---|---|---|
| Potato | 640×360 | 60 FPS |
| Low | 1024×576 | 40–60 FPS |
| Medium | 1366×768 | 25–40 FPS |
| Ultra | 1366×768 | Best effort |

These are engineering targets, NOT guaranteed performance.

Target RAM consumption:
- Idle: below 200 MB where practical.
- Typical gameplay: below 500 MB.
- Heavy scenes: preferably below 1 GB.

Do not sacrifice correctness or stability solely to meet memory targets.

---

# 2. OPENGL 2.1 / GLSL 1.20

Every desktop shader MUST begin with:

```glsl
#version 120
```

Use GLSL 1.20 syntax:

- `attribute` for vertex shader inputs.
- `varying` for vertex-to-fragment interpolation.
- `uniform` for shader parameters.
- `texture2D()` for 2D texture sampling.
- `gl_FragColor` or compatible fragment outputs.

Forbidden:
- `#version 130` or newer.
- `layout(location = ...)`.
- Modern shader `in/out` declarations.
- `texture()` in desktop GLSL 1.20.
- Modern GLSL buffer interfaces.

## Vertex buffers

Use:
- VBOs.
- IBO/EBOs.
- `glVertexAttribPointer`.
- `glEnableVertexAttribArray`.
- `glGetAttribLocation` or `glBindAttribLocation` before program linking.

Do not require:
- `glGenVertexArrays`.
- `glBindVertexArray`.
- Vertex Array Objects.

For OpenGL 2.1, configure vertex attributes explicitly when required.

## Renderer requirements

Implement:
- Frustum culling.
- Distance culling.
- Static geometry batching.
- Texture atlases.
- Mipmapping.
- LOD.
- Resource caching.
- Dynamic render distance.
- Optional resolution scaling.
- Lightweight fog.
- Configurable shadows.

Minimize:
- Draw calls.
- Shader state changes.
- Texture bindings.
- CPU-to-GPU transfers.
- Overdraw.
- Allocations inside the render loop.

Never upload unchanged mesh data every frame.

---

# 3. RUST STABILITY & ERROR HANDLING

Use stable Rust.

Critical operations MUST return meaningful errors.

Never use `.unwrap()` or `.expect()` on fallible external operations such as:
- OpenGL function loading.
- Shader compilation/linking.
- File I/O.
- SDL2 initialization.
- Audio initialization.
- Asset loading.
- Save file parsing.
- Network connections.

Use `Result<T, E>` and propagate errors with `?`.

Log failures using `eprintln!`, `log`, or `tracing`.

When shader compilation fails:
1. Query compile status.
2. Retrieve shader info log.
3. Report shader name and stage.
4. Log the driver error.
5. Release initialized resources where possible.
6. Exit gracefully or use an explicitly supported fallback.

Never intentionally crash the process for a recoverable failure.

During development, use:

```bash
RUST_BACKTRACE=1 cargo run
```

Before declaring a stage complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo build --release
```

If a command fails, fix the error or document the blocker.

Do not fabricate test results.

---

# 4. ENGINE ARCHITECTURE

Use a modular Cargo workspace.

Suggested structure:

```text
jailbreak-rs/
├── AGENTS.md
├── Cargo.toml
├── crates/
│   ├── engine_core/
│   ├── platform_sdl/
│   ├── renderer_gl21/
│   ├── renderer_gles2/
│   ├── physics/
│   ├── world/
│   ├── vehicles/
│   ├── construction/
│   ├── ai/
│   ├── gameplay/
│   ├── economy/
│   ├── ui/
│   ├── audio/
│   ├── networking/
│   └── persistence/
├── assets/
├── configs/
├── benchmarks/
└── docs/
```

Reuse existing working modules instead of rewriting them.

The core engine MUST NOT directly depend on desktop OpenGL or Linux-specific APIs.

Separate:
- Platform.
- Rendering.
- Simulation.
- Gameplay.
- UI.
- Persistence.

Do not place all gameplay systems inside `main.rs`.

---

# 5. SDL2 PLATFORM LAYER

SDL2 is a platform library, NOT the game engine.

Use it for:
- Window creation.
- OpenGL context creation.
- Keyboard input.
- Mouse input.
- Game controllers.
- Touch events.
- Audio integration where appropriate.

SDL2 platform code must remain separate from gameplay.

Create platform-independent input actions:

```text
MoveForward
MoveBackward
MoveLeft
MoveRight
Jump
Interact
EnterVehicle
ExitVehicle
UsePhysgun
RotateObject
OpenBuildMenu
OpenInventory
```

Map physical devices to these actions.

Do not hardcode keyboard scan codes inside gameplay logic.

---

# 6. ANDROID COMPATIBILITY

The architecture MUST allow a future Android APK.

Target:
- ARM64.
- Android-compatible Rust toolchain.
- OpenGL ES 2.0 baseline.
- Optional OpenGL ES 3.0 optimizations.

IMPORTANT:

Desktop OpenGL 2.1 is NOT equivalent to OpenGL ES 2.0.

Create a separate GLES renderer.

Desktop shader:

```glsl
#version 120
```

Android shader:

```glsl
#version 100
precision mediump float;
```

Vertex and fragment shaders must use the correct syntax for their respective backend.

For GLES2, account for:
- Precision qualifiers.
- Different extension availability.
- Texture format limitations.
- Framebuffer compatibility.
- Attribute limits.
- Shader precision limits.

Never send desktop GLSL 1.20 shaders directly to GLES2 without adaptation.

## Mobile controls

Implement:
- Virtual movement joystick.
- Touch camera rotation.
- Jump button.
- Interaction button.
- Vehicle throttle/brake.
- Steering controls.
- Physgun controls.
- Building UI.
- Inventory UI.
- Adjustable touch sensitivity.

Mobile UI must scale to different screen sizes.

## Mobile optimization

Use:
- Lower render resolution.
- Dynamic resolution.
- Reduced vegetation.
- Lower physics object budgets.
- Simplified shadows.
- Aggressive LOD.
- Reduced NPC update frequency.
- Texture compression where supported.

Android packaging must be implemented and tested separately.

Do not claim Android compatibility merely because Rust code compiles for ARM64.

---

# 7. PHYSICS SANDBOX — GMOD-STYLE

Jailbreak RS must support a modular physics construction system.

The goal is to let players disassemble, modify and rebuild vehicles and structures.

## Physgun

Implement:
- Raycast object selection.
- Object pickup.
- Distance adjustment.
- Rotation.
- Freeze/unfreeze.
- Controlled release.
- Collision-safe placement.
- Visual selection indicator.

Physgun must work with movable rigid bodies.

## Toolgun

Tools:
- Weld.
- Rope.
- Hinge.
- Slider.
- Motor.
- Axis.
- Spring.
- Thruster.
- Collision toggle.
- Material editor.
- Color editor.
- Duplicator.

Each tool must support undo when feasible.

## Vehicle construction

Vehicles consist of:
- Chassis.
- Wheels.
- Suspension.
- Engine.
- Transmission.
- Steering.
- Brakes.
- Body panels.
- Seats.
- Fuel or energy storage.
- Optional electronics.

A vehicle must not be only a decorative model with scripted movement.

Allow players to:
- Remove parts.
- Replace engines.
- Change wheels.
- Build custom chassis.
- Add armor.
- Add propulsion.
- Save blueprints.
- Spawn saved creations.

## Physics optimization

Use:
- Fixed timestep.
- Broadphase collision detection.
- Simplified collision shapes.
- Sleeping rigid bodies.
- Spatial partitioning.
- Compound rigid bodies.
- Constraint islands.
- Object pooling where useful.

Welded rigid assemblies should be merged or otherwise optimized where physically valid.

Preserve articulated joints.

Do not merge components that require independent movement.

Set configurable physics budgets.

Never assume hundreds of active constrained bodies will run at 60 FPS on i7-620M.

---

# 8. VEHICLE PHYSICS

Vehicles must have:
- Mass.
- Center of mass.
- Acceleration.
- Braking.
- Steering.
- Suspension.
- Tire grip.
- Collision response.
- Damage state.
- Camera follow.
- Multiple seats.

Use a fixed simulation timestep.

Do not tie vehicle acceleration or handling to render FPS.

Support both:
- Optimized arcade vehicles.
- Advanced modular physics vehicles.

The two systems should share gameplay interfaces without requiring identical internal simulation complexity.

---

# 9. WORLD & MAP

The world should contain:
- Prison.
- Police station.
- City.
- Secondary city.
- Roads.
- Highways.
- Tunnels.
- Bridges.
- Airport.
- Criminal base.
- Garages.
- Robbery locations.
- Mountains.
- Desert.
- Water.

World coordinates and important locations must be centralized.

Use data-driven map definitions.

Avoid hardcoding building positions across multiple modules.

Implement:
- Chunk streaming.
- World partitioning.
- LOD.
- Collision loading.
- Terrain height queries.
- Road navigation graph.
- AI navigation data.

Static buildings should be batched when practical.

Important interactive structures should retain separate collision and interaction state.

---

# 10. UI / HUD

Use a custom lightweight UI renderer compatible with OpenGL 2.1 and GLES2.

Required interfaces:
- Main menu.
- Team selection.
- HUD.
- Inventory.
- Garage.
- Vehicle customization.
- Build menu.
- Physgun tool menu.
- Full map.
- Minimap.
- Settings.
- Save/load interface.

UI should be rendered independently of internal 3D render resolution.

For example:
- World rendering: 640×360.
- Display resolution: 1366×768.
- UI: native display resolution.

Implement resolution-independent coordinates.

Avoid allocating UI geometry unnecessarily every frame.

Use cached text and icon atlases where appropriate.

---

# 11. DATA-DRIVEN GAMEPLAY

Centralize:
- Vehicle specifications.
- Prices.
- Weapon statistics.
- Robbery rewards.
- NPC parameters.
- Map coordinates.
- Physics material properties.
- Construction part definitions.

Prefer versioned JSON, RON, TOML or appropriate binary asset formats.

Example:

```toml
[starter_sedan]
price = 0
mass_kg = 1250.0
max_speed_kmh = 165.0
engine_power_kw = 95.0
wheel_count = 4
```

Values are initial gameplay configuration, not claims about a real vehicle.

Do not duplicate configuration constants across files.

Validate configuration values during loading.

---

# 12. PERFORMANCE PROFILING

Implement an in-game performance overlay.

Required metrics:
- FPS.
- Frame time.
- CPU update time.
- Physics time.
- AI time.
- Render submission time.
- Draw calls.
- Triangle count.
- Active chunks.
- Active vehicles.
- Active NPCs.
- Active rigid bodies.
- Sleeping rigid bodies.
- RAM usage.
- Graphics API version.
- GPU renderer string.

Do not confuse CPU render submission time with actual GPU execution time.

Use GPU timer queries only when supported by the driver.

Add an optional uncapped benchmark mode.

Provide reproducible benchmark scenes:
1. Empty map.
2. Dense city.
3. High-speed driving.
4. Police pursuit.
5. Physics construction.
6. Large vehicle collision.
7. Mixed AI + physics stress test.

Record actual results on the target hardware.

---

# 13. SIMULATION SCHEDULING

Use separate update frequencies when appropriate.

Suggested starting points:
- Rendering: variable rate.
- Physics: fixed 60 Hz, adjustable.
- Near AI: 10–20 Hz.
- Distant AI: 1–5 Hz.
- Economy: event-driven or low-frequency.
- World streaming: asynchronous preparation.

These are starting values and must be profiled.

Never mutate shared simulation state unsafely.

Avoid unbounded task queues.

Use bounded workloads to prevent frame spikes.

Keep expensive asset decoding and mesh generation off the render thread when possible.

All OpenGL operations must execute in a context-valid thread.

---

# 14. SAVE SYSTEM

Persist:
- Player position.
- Team.
- Money.
- Owned vehicles.
- Vehicle customization.
- Construction blueprints.
- World modifications.
- Inventory.
- Missions.
- Settings.

Use versioned save formats.

Implement:
- Atomic save replacement where supported.
- Corruption detection.
- Graceful recovery.
- Schema migration.
- Backup saves.

Never overwrite a valid save with corrupted or incomplete data.

---

# 15. NETWORKING

Design the engine for future multiplayer.

Use server-authoritative logic for:
- Economy.
- Arrests.
- Damage.
- Robbery completion.
- Ownership.
- Shared physics interactions.

Support:
- Client prediction where useful.
- Entity interpolation.
- Snapshot synchronization.
- Interest management.
- Bandwidth budgets.
- Reconciliation.

Do not attempt to synchronize every physics object every render frame.

Prioritize nearby, active and gameplay-relevant objects.

Networking must not block the main render loop.

Do not claim multiplayer is complete without real client/server tests.

---

# 16. REGRESSION PREVENTION

Before changing any major subsystem:

1. Inspect existing implementation.
2. Identify public interfaces.
3. Identify dependent modules.
4. Preserve existing gameplay.
5. Implement changes incrementally.
6. Run compilation and tests.
7. Perform runtime validation when possible.
8. Update documentation.

Never silently delete working functionality.

Never replace a complete subsystem with a placeholder just to make the project compile.

If a refactor is necessary, preserve behavior through tests.

---

# 17. CODEX EXECUTION POLICY

Work in small, verifiable milestones.

For every milestone:
1. Explain the implementation objective briefly.
2. Inspect relevant code.
3. Modify actual source files.
4. Compile.
5. Run automated tests.
6. Fix errors.
7. Summarize changes.
8. Record known limitations.

If hardware-specific runtime testing is unavailable, state that explicitly.

Do not invent FPS measurements.

Do not claim that rendering, physics or Android packaging works without appropriate testing.

Use Git branches or worktrees for parallel development.

Do not let multiple agents overwrite the same files concurrently.

---

# 18. IMPLEMENTATION PRIORITIES

Priority 1:
Stable Rust + SDL2 + OpenGL 2.1 desktop build.

Priority 2:
Playable Jailbreak core: prison, city, player, cars, police, robberies.

Priority 3:
Physgun, Toolgun, modular vehicles, construction blueprints.

Priority 4:
Advanced vehicle physics, AI and expanded world systems.

Priority 5:
GLES2 renderer, touch controls, Android APK.

Priority 6:
Multiplayer, server-authoritative physics and cross-platform play.

Priority 7:
Optimization, content expansion and final polish.

Every priority must preserve compatibility with the target hardware.

---

# 19. NON-NEGOTIABLE RULES

- Never require modern GPU features.
- Never break OpenGL 2.1 compatibility.
- Never confuse desktop GL with GLES.
- Never introduce unnecessary heavyweight dependencies.
- Never block rendering with expensive world generation.
- Never invent successful test results.
- Never discard working code without justification.
- Never copy proprietary Roblox Jailbreak assets.
- Never sacrifice save data integrity.
- Never silently disable gameplay features to increase FPS.
- Always prefer measured optimization over assumptions.

# FINAL ENGINEERING OBJECTIVE

Build a modular Rust open-world engine capable of supporting:

- Jailbreak-style crime gameplay.
- Garry's Mod-style physics tools.
- Besiege-style vehicle construction.
- Large open-world exploration.
- AI-driven police pursuits.
- Future Android support.
- Future multiplayer.

The engine must remain usable on Intel HD Ironlake through scalable graphics and simulation complexity.

**The ultimate objective is not maximum graphical fidelity. It is maximum systemic freedom per millisecond of computation.**
