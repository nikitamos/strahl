# Overview
This crate is designed to import assets into `strahl` engine.

# Assets format
`strahl` uses assets of three types:
1. Materials, stored as ZIP archives.
2. Geometries, stored as glTF scenes.
3. Cubemaps, stored as PNG images.

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

## Validity
The glTF file must satisfy the following conditions:
1. It has exactly one mesh.
2. The `primitives` field of the mesh has the following form:
```jsonc
{
    /* ... */
    "primitives" : [
        {
            "attributes": {
                "POSITION":   /* accessor of float3 */,
                "NORMAL":     /* accessor of float3 */,
                "TEXCOORD_0": /* accessor of float2 */
                // other attributes are ignored, but not disallowed
            },
            "indices": /* accessor of scalar integral */
        }
    ],
    /* ... */
}
```
3. Used accessors are not sparse.
4. All attributes are stored in the same buffer.

The following command may be used to validate the glTF file:
```
cargo r -- validate-gltf path/to/cubemap/geometry.gltf
```

# Cubemaps
Cubemap is stored as a set of six PNG images located at the
same directory. Each image is mapped to specific
face of the cube.

Consider a unit cube centered at the origin $O$. Each face is
intersected by single coordinate ray. Each image file is named
after ray intersecting the corresponding face, as shown
in table below:

<center>

|   File name   | Corresponding ray |
|---------------|-------------------|
| `x_plus.png`  |      $ Ox $       |
| `x_minus.png` |      $ -Ox $      |
| `y_plus.png`  |      $ Oy $       |
|       ...     |      ...          |
</center>

## Validity
Images representing a valid cubemap must satisfy the following conditions:
1. All images have the same dimensions.
2. For each image, its height is equal to its width.
3. Each image is stored in RGBA8 format.

The following command may be used to validate the cubemap:
```
cargo r -- validate-cubemap path/to/cubemap/dir [--transcode]
```

The `--transcode` option may be passed to allow transcoding image files
to RGBA8 if they are in wrong format.
