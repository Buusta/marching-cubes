mod marching_cubes;
mod tri_tables;

pub use tri_tables::*;

use godot::prelude::*;

struct MarchingCubes;

#[gdextension]
unsafe impl ExtensionLibrary for MarchingCubes {}