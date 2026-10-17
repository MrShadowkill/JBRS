# JBRS GLSL shader packs

Put shader packs in `shaders/packs/<PackName>/`. The in-game Settings menu
lists usable packs and switches them while playing. Settings are written into
the existing JBRS save file, preserving existing game progress. The built-in
renderer shaders remain the safe fallback. Menus and interface keep their
built-in rendering.

User pack directories are `C:\\JBRS\\shaders\\packs` on Windows,
`~/JBRS/shaders/packs` on Linux, and `/storage/emulated/0/JBRS/shaders/packs`
on Android. You can also set `JBRS_SHADER_DIR` to a shader directory; packs
belong in its `packs/` subdirectory. Android APK builds include repository
packs as assets. External shared storage access depends on Android storage
permissions and may vary by device.

## Desktop OpenGL 2.1

Put the pair in `shaders/packs/<PackName>/desktop/`, for example:

```text
shaders/packs/Example/desktop/world.vert
shaders/packs/Example/desktop/world.frag
```

Use GLSL 1.20 (`#version 120`) and the fixed-function OpenGL 2.1 interface in
`renderer_gl21/shaders/`.

## GLES2 (Android)

Put a separate pair in `shaders/packs/<PackName>/gles2/`, for example:

```text
shaders/packs/Example/gles2/world.vert
shaders/packs/Example/gles2/world.frag
```

Use GLSL ES 1.00 (`#version 100`) and the interface in
`renderer_gles2/shaders/`. Never pass desktop GLSL directly to GLES2.

## Folder or ZIP

You can use a `<PackName>` directory or `<PackName>.zip`; archives are read
without extraction. For ZIP packs, include both `desktop/` and `gles2/`
subdirectories. Matching vertex/fragment suffixes include `.vert`,
`.vert.glsl`, `.vertex`, `.vs`, `.vsh` and `.frag`, `.frag.glsl`, `.fragment`,
`.fs`, `.fsh`; `world.vert` and `world.frag` are preferred.

Shader sources are limited to 1 MiB each and ZIPs to 64 MiB. If compilation
fails, JBRS logs the driver error and retains or restores the built-in pair.

## Golden Hour example

`packs/Golden_Hour_v2/` demonstrates desktop and GLES2 pairs with stylized
day/night color and distance fog. It does not cast geometry shadows.
