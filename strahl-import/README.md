# Overview
This crate is designed to import assets into `strahl` engine.

# Assets format
`strahl` uses assets of two types:
1. Materials, stored as ZIP archives.
2. Geometries, stored as glTF scenes.

# Material
## Archive format
Material is stored in a ZIP archive of the following structure:
* `/surface`
    * `roughness.png`
    * `specular.png`
    * `glossy.png`
    * `diffuse.png`
    * `emission.png`
    * `normal.png`
* `/metadata.toml`

As seen, textures are in the png format. Support of
the KTX2 format may be added in future.

The `metadata.toml` file specifies whether the texture or uniform color
is used by each component of BSDF. Thus every texture is optional
and may be replaced by color declaration in `metadata.toml`.

## Creating archives
For purposes of creating archives and transcoding png to ktx2
this create provides an utility binary. It builds an
archive from material description written in TOML format.

### Material.toml
Textures or colors for each BSDF component are specified
in a `material.toml` file. If any BSDF component is omitted, it defaults
to `rgba(0, 0, 0, 255)`.
```toml
[textures]
# The paths are resolved relative to directory containing material.toml
diffuse="Lava diffuse.png"
emission="Lava emission.png"
glossy="Lava glossy.png"
roughness="Lava roughness.png"
normal="Lava normal.png"
# Alternatively, RGBA color may be specified
specular = {r=0, g=0, b=0, a=255}
```

In future more options related to the KTX compression may be added.

### Writing zip file
Once the `material.toml` is set up, the archive can be created
using the command
```
cargo r -- create-material path/to/material.toml path/to/output.zip
```

# Geometry
Geometry, its UV map, vertex normals are stored in a glTF file.
