# JBRS GLSL shader packs

Place a custom shader pair here to override the built-in world shader. JBRS
ships its own renderer and does not execute arbitrary shader files: the pair
must use the attributes, uniforms, and varyings expected by the world renderer.
The menus and interface continue to use their built-in shaders.

## Desktop OpenGL 2.1

Put the pair in `shaders/desktop/`, for example:

```text
shaders/desktop/world.vert
shaders/desktop/world.frag
```

Use GLSL 1.20 (`#version 120`) and the fixed-function OpenGL 2.1 interface used
by the built-in desktop world shaders. Keep the same vertex outputs and
fragment inputs as the built-in files in `renderer_gl21/shaders/`.

## GLES2 (Android)

Put the GLES pair in `shaders/gles2/`, for example:

```text
shaders/gles2/world.vert
shaders/gles2/world.frag
```

Use GLSL ES 1.00 (`#version 100`). Keep the built-in shader interface from
`renderer_gles2/shaders/`: vertex attributes `position`, `color`, `texcoord`;
uniforms `model_view` and `projection`; and the world fragment shader's
`atlas`, `daylight`, `opacity`, `fog_end`, `detail_amount`, `mesh_mode`, and
`model_tint` uniforms with `col`, `uv`, and `dist` varyings.

## Folder or ZIP

You can place a folder containing a matching vertex/fragment pair under this
directory, or put a `.zip` archive here. Archives are read without extraction.
For Android APK assets, name the archive `shaderpack.zip`, or put the pair in
`gles2/` directly.
Supported vertex suffixes include `.vert`, `.vert.glsl`, `.vertex`, `.vs`, and
`.vsh`; fragment suffixes include `.frag`, `.frag.glsl`, `.fragment`, `.fs`,
and `.fsh`. `world.vert` and `world.frag` are preferred when a pack contains
multiple pairs. For a pack with both renderers, use `desktop/` and `gles2/`
subfolders in the archive.

An external pair is compiled at startup. If it does not compile or link, JBRS
logs the error and uses its built-in shader pair. Set `JBRS_SHADER_DIR` to a
different pack directory to load it from elsewhere. Android APKs include the
contents of this directory as assets; replace the assets and rebuild the APK
to ship a different pack.
