@tool
extends Node

@export_category("Generation settings")
@export_range(1, 128, 1) var resolution: int = 16:
	set(p_resolution):
		#if p_resolution != resolution:
		resolution = p_resolution
		_generate_points_values()

@export var surface_level: float = 0.2

@export_category("Noise settings")
@export var frequency: float = 0.01:
	set(p_frequency):
		if p_frequency != frequency:
			frequency = p_frequency
			_initialize_noise()
			_generate_points_values()

@export var noise_type: FastNoiseLite.NoiseType = FastNoiseLite.TYPE_SIMPLEX:
	set(p_noise_type):
		if p_noise_type != noise_type:
			noise_type = p_noise_type
			_initialize_noise()
			_generate_points_values()

@export var noise_offset: Vector3 = Vector3.ZERO:
	set(p_noise_offset):
		if p_noise_offset != noise_offset:
			noise_offset = p_noise_offset
			_initialize_noise()
			_generate_points_values()

@export var noise_seed: int = 69420:
	set(p_noise_seed):
		if p_noise_seed != noise_seed:
			noise_seed = p_noise_seed
			_initialize_noise()
			_generate_points_values()

@export_category("Debug settings")
@export var debug_sphere_radius: float = 0.1
@export var debug_show_surface_level: bool = true
@export var show_debug_spheres: bool = true
@export var refresh: bool = false:
	set(p_refresh):
		refresh = false
		var timmy: int = Time.get_ticks_usec()
		_initialize_noise()
		_generate_points_values()
		_generate_marching_mesh()
		var timmy2: int = Time.get_ticks_usec()
		print((timmy2-timmy)/1000.0)


var noise: FastNoiseLite = FastNoiseLite.new()
var points: Array[Vector3]
var values: PackedFloat32Array = PackedFloat32Array()

var mesh_instance: MeshInstance3D = MeshInstance3D.new()

func _initialize_noise() -> void:
	noise.frequency = frequency
	noise.noise_type = noise_type
	noise.seed = noise_seed
	noise.fractal_octaves = 12

	print(noise.fractal_type)

func _generate_points_values() -> void:
	points.clear()
	values.clear()

	var total_points: int = (resolution) * (resolution) * (resolution)
	points.resize(total_points)
	values.resize(total_points)

	var loop_idx: int = 0
	for z: int in range(resolution):
		for y: int in range(resolution):
			for x: int in range(resolution):
				var pos: Vector3 = Vector3(x, y, z)
				points[loop_idx] = pos
				values[loop_idx] = noise.get_noise_3d(x, y, z)
				loop_idx += 1

func _process(_delta: float) -> void:
	if  show_debug_spheres:
		for i: int in len(points):
			var point: Vector3 = points[i]
			var value: float = values[i]

			if debug_show_surface_level:
				if not value < surface_level:
					DebugDraw3D.draw_sphere(point, debug_sphere_radius, Color(value, value, value, 1.0))
			else:
				DebugDraw3D.draw_sphere(point, debug_sphere_radius, Color(value, value, value, 1.0))

func _generate_marching_mesh() -> void:
	var verts: PackedVector3Array = _generate_vertex_array()

	var st: SurfaceTool = SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	for vert: Vector3 in verts:
		st.add_vertex(vert)

	st.generate_normals()
	var mesh: ArrayMesh = st.commit()
	
	if not mesh_instance:
		mesh_instance = MeshInstance3D.new()

	if not mesh_instance.get_parent():
		add_child(mesh_instance)

	mesh_instance.mesh = mesh


func sample(x:int, y:int, z:int) -> float:
	return values[x + y*resolution + z*resolution*resolution]

func _generate_vertex_array() -> PackedVector3Array:
	var num_cubes: int = (resolution - 1) * (resolution - 1) * (resolution - 1)
	var max_verts: int = num_cubes * 15  # worst case 5 triangles per cube
	var vert_array: PackedVector3Array = PackedVector3Array()
	vert_array.resize(max_verts)
	var vert_idx: int = 0

	for y: int in range(resolution - 1):
		for z: int in range(resolution - 1):
			for x: int in range(resolution - 1):
				# sample cube corners
				var cube_values: Array[float] = [
					sample(x+1, y,   z),
					sample(x+1, y,   z+1),
					sample(x,   y,   z+1),
					sample(x,   y,   z),
					sample(x+1, y+1, z),
					sample(x+1, y+1, z+1),
					sample(x,   y+1, z+1),
					sample(x,   y+1, z),
				]

				# --- cube index timer ---
				var cube_index: int = 0
				for i: int in range(8):
					if cube_values[i] > surface_level:
						cube_index |= 1 << i

				# --- vert list timer ---
				var edge_mask: int = CubesTables.EdgeTable[cube_index]
				var vert_list: Array[Vector3] = []
				vert_list.resize(12)
				for i: int in range(12):
					if edge_mask & (1 << i):
						var idx0: int = CubesTables.EdgeIndexTable[i][0]
						var idx1: int = CubesTables.EdgeIndexTable[i][1]

						var pos0: Vector3 = CubesTables.CubeTable[idx0] + Vector3(x, y, z)
						var pos1: Vector3 = CubesTables.CubeTable[idx1] + Vector3(x, y, z)
						var val0: float = cube_values[idx0]
						var val1: float = cube_values[idx1]

						if abs(surface_level - val0) < 0.00001:
							vert_list[i] = pos0
						elif abs(surface_level - val1) < 0.00001:
							vert_list[i] = pos1
						elif abs(val0 - val1) < 0.00001:
							vert_list[i] = pos0
						else:
							var mu: float = (surface_level - val0) / (val1 - val0)
							vert_list[i] = pos0.lerp(pos1, mu)

				# --- triangle verts timer ---
				var tri_start: int = CubesTables.TriStart[cube_index]
				var tri_end: int = CubesTables.TriStart[cube_index + 1]
				for i: int in range(tri_start, tri_end, 3):
					var a: Vector3 = vert_list[CubesTables.TriTableFlat[i]]
					var b: Vector3 = vert_list[CubesTables.TriTableFlat[i+1]]
					var c: Vector3 = vert_list[CubesTables.TriTableFlat[i+2]]

					# flip winding
					vert_array[vert_idx] = a
					vert_array[vert_idx+1] = c
					vert_array[vert_idx+2] = b
					vert_idx += 3

	# trim excess
	vert_array.resize(vert_idx)
	return vert_array


func _get_cube_verts(cube_values: Array[float], offset: Vector3) -> Array[Vector3]:
	var cube_idx: int = _get_cube_index(cube_values)

	var vert_list: Array[Vector3] = _get_vert_list(cube_values, cube_idx, offset)

	var triangle_verts: Array[Vector3] = _get_triangle_verts(cube_idx, vert_list)

	return triangle_verts

func _get_cube_index(cube_values: Array[float]) -> int:
	var cube_index: int = 0
	for i: int in range(8):
		if cube_values[i] > surface_level:
			cube_index |= 1 << i

	return cube_index

func _get_vert_list(cube_values: Array[float], cube_idx: int, offset: Vector3) -> Array[Vector3]:
	var vert_list: Array[Vector3] = []
	vert_list.resize(12)

	var edge_mask: int = CubesTables.EdgeTable[cube_idx]
	for i: int in range(12):
		if edge_mask & (1 << i):
			vert_list[i] = _get_vert_position(i, cube_values, offset)

	return vert_list

func _get_vert_position(edge_idx: int, cube_values: Array[float], offset: Vector3) -> Vector3:
	var edge_verts: Array = CubesTables.EdgeIndexTable[edge_idx]
	var index_0: int = edge_verts[0]
	var index_1: int = edge_verts[1] 

	var position_0: Vector3 = CubesTables.CubeTable[index_0] + offset
	var position_1: Vector3 = CubesTables.CubeTable[index_1] + offset

	var value_0: float = cube_values[index_0]
	var value_1: float = cube_values[index_1]

	if abs((surface_level-value_0)) < 0.00001:
		return position_0
	if abs(surface_level-value_1) < 0.00001:
		return position_1
	if abs(value_0-value_1) < 0.00001:
		return position_0

	var mu: float = (surface_level-value_0) / (value_1-value_0)

	var vert: Vector3 = position_0.lerp(position_1, mu)

	return vert

func _get_triangle_verts(cube_idx: int, vert_list: Array[Vector3]) -> Array[Vector3]:
	var start: int = CubesTables.TriStart[cube_idx]
	var end: int = CubesTables.TriStart[cube_idx + 1]

	var count: int = end - start

	var out: Array[Vector3] = []
	out.resize(count)

	var t: int = 0

	var i: int = start
	while i < end:

		var a: Vector3 = vert_list[CubesTables.TriTableFlat[i]]
		var b: Vector3 = vert_list[CubesTables.TriTableFlat[i + 1]]
		var c: Vector3 = vert_list[CubesTables.TriTableFlat[i + 2]]

		# flip winding
		out[t]   = a
		out[t+1] = c
		out[t+2] = b

		t += 3
		i += 3

	return out
