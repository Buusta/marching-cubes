mod marching_cubes;

use godot::prelude::*;

struct MarchingCubes;

#[gdextension]
unsafe impl ExtensionLibrary for MarchingCubes {}