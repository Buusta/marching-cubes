@tool
extends Node3D

@export var show_debug_spheres: bool = false
@export var show_surface_level: bool = true
@export var debug_sphere_size: float = 0.2
@export var marching_cubes: MarchingCubes

var vals: PackedFloat32Array
var resolution: int
var surface_level: float

func _enter_tree() -> void:
	marching_cubes.refreshed.connect(_refreshed)

func _process(_delta: float) -> void:
	surface_level = marching_cubes.surface_level

	if not show_debug_spheres:
		return

	var loop_idx: int = 0
	for y: int in range(resolution):
		for x: int in range(resolution):
			for z: int in range(resolution):
				var val: float = vals[loop_idx]
				if show_surface_level:
					if not val < surface_level:
						DebugDraw3D.draw_sphere(Vector3(x, y, z), debug_sphere_size, Color(val, val, val))
					loop_idx += 1
					continue
				
				DebugDraw3D.draw_sphere(Vector3(x, y, z), debug_sphere_size, Color(val, val, val))
				loop_idx += 1
	
func _refreshed() -> void:
	vals = marching_cubes.get_values()
	resolution = marching_cubes.resolution
