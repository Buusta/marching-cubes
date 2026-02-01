mod marching_cubes;
mod tri_tables;

pub use tri_tables::*;
pub use marching_cubes::MarchingCubes;

use godot::prelude::*;

#[gdextension]
unsafe impl ExtensionLibrary for MarchingCubes {}