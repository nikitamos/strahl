# Destination
This crate is designed to import assets into `strahl` engine in backend-agnostic manner.

# Assets format
`strahl` uses assets of two types:
1. Materials, stored in ZIP archives.
2. Geometries, stored as glTF scenes.

# Material
Material is stored in a ZIP archive of the following structure:
* `/surface`
    * `roughness.ktx2`
    * `specular.ktx2`
    * `glossy.ktx2`
    * `diffuse.ktx2`
    * `emission.ktx2`
* `/normal.ktx2`

# Geometry
Geometry, its UV map, vertex normals are stored in a glTF file.
