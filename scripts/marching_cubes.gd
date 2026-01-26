@tool
extends Node

@export_category("Generation settings")
@export_range(1, 128, 1) var resolution: int = 16:
	set(p_resolution):
		if p_resolution != resolution:
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
		var tim: int = Time.get_ticks_msec()
		_initialize_noise()
		_generate_points_values()
		_generate_marching_mesh()
		var tim2: int = Time.get_ticks_msec()
		print('total time: ', tim2-tim)


var noise: FastNoiseLite = FastNoiseLite.new()
var points: Array[Vector3]
var values: Array[float]

var mesh_instance: MeshInstance3D = MeshInstance3D.new()

func _initialize_noise() -> void:
	noise.frequency = frequency
	noise.noise_type = noise_type
	noise.seed = noise_seed

func _generate_points_values() -> void:
	var tim: int = Time.get_ticks_msec()
	points.clear()
	values.clear()
	for y: float in range(resolution - 1):
		for z: float in range(resolution - 1):
			for x: float in range(resolution - 1):
				for point: Vector3 in CubesTables.CubeTable:
					var offset_point: Vector3 = point + Vector3(x, y, z)

					var noise_value: float = noise.get_noise_3d(offset_point.x, offset_point.y, offset_point.z)

					points.append(offset_point)
					values.append(noise_value)

	var tim2: int = Time.get_ticks_msec()
	print('points + value time: ', (tim2-tim))

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

func _generate_vertex_array() -> PackedVector3Array:
	var idx: int = 0
	var vert_array: PackedVector3Array = PackedVector3Array()

	var cube_verts_time: int = 0

	cube_idx_tim = 0
	cube_vert_list_tim = 0
	cube_tri_verts_tim = 0
	tri_table_tim = 0
	tri_append_tim = 0

	for y: float in range(resolution - 1):
		for z: float in range(resolution - 1):
			for x: float in range(resolution - 1):
				var offset: Vector3 = Vector3(x, y, z)
				var cube_values: Array[float] = values.slice(idx, idx + 8)

				var tim3: int = Time.get_ticks_usec()
				var cube_verts: Array[Vector3] = _get_cube_verts(cube_values, offset)
				var tim4: int = Time.get_ticks_usec()
				cube_verts_time += (tim4-tim3)

				vert_array.append_array(cube_verts)
				idx += 8

	print("verts time: ", float(cube_verts_time)/1000.0)
	print("cube idx time: ", float(cube_idx_tim)/1000.0)
	print("cube vert list time: ", float(cube_vert_list_tim)/1000.0)
	print("cube triangle verts time: ", float(cube_tri_verts_tim)/1000.0)

	return vert_array

var cube_idx_tim: int = 0
var cube_vert_list_tim: int = 0
var cube_tri_verts_tim: int = 0
var tri_table_tim: int = 0
var tri_append_tim: int = 0

func _get_cube_verts(cube_values: Array[float], offset: Vector3) -> Array[Vector3]:
	var cube_idx_start: int = Time.get_ticks_usec()
	var cube_idx: int = _get_cube_index(cube_values)
	var cube_idx_end: int = Time.get_ticks_usec()
	cube_idx_tim += (cube_idx_end-cube_idx_start)

	var vert_list_start: int = Time.get_ticks_usec() 
	var vert_list: Array[Vector3] = _get_vert_list(cube_values, cube_idx, offset)
	var vert_list_end: int = Time.get_ticks_usec()
	cube_vert_list_tim += (vert_list_end - vert_list_start)

	var tri_list_start: int = Time.get_ticks_usec() 
	var triangle_verts: Array[Vector3] = _get_triangle_verts(cube_idx, vert_list)
	var tri_list_end: int = Time.get_ticks_usec() 
	cube_tri_verts_tim += (tri_list_end-tri_list_start)

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

#func _get_triangle_verts(cube_idx: int, vert_list: Array[Vector3]) -> Array[Vector3]:
	#var tri_table_start: int = Time.get_ticks_usec() 
	#var tri_table: Array = CubesTables.TriTable[cube_idx]
	#var tri_table_end: int = Time.get_ticks_usec()
	#tri_table_tim += (tri_table_end-tri_table_start)
	#var triangle_verts: Array[Vector3] = []
#
	#var tri_append_start: int = Time.get_ticks_usec() 
	#for i: int in range(0, tri_table.size()-1, 3):
		#var tri_indices: Array = tri_table.slice(i, i+3)
		#var verts: Array[Vector3] = []
		#for idx in tri_indices:
			#verts.append(vert_list[idx])
#
		### flip winding
		#triangle_verts.append_array(verts)
		###triangle_verts.append(c)
		###triangle_verts.append(b)
#
	#var tri_append_end: int = Time.get_ticks_usec() 
	#tri_append_tim += (tri_append_end-tri_append_start)
#
	#return triangle_verts

func _get_triangle_verts(cube_idx: int, vert_list: Array[Vector3]) -> Array[Vector3]:
	var tri_table: Array = CubesTables.TriTable[cube_idx]

	# Count valid indices (skip -1 if present)
	var tri_count: int = 0
	while not tri_table[tri_count] == -1:
		tri_count += 1

	var triangle_verts: Array[Vector3] = []
	triangle_verts.resize(tri_count)

	var t: int = 0
	for i: int in range(0, tri_count, 3):
		var a: Vector3 = vert_list[tri_table[i]]
		var b: Vector3 = vert_list[tri_table[i+1]]
		var c: Vector3 = vert_list[tri_table[i+2]]

		# Flip winding directly while writing
		triangle_verts[t]   = a
		triangle_verts[t+1] = c
		triangle_verts[t+2] = b
		t += 3

	return triangle_verts
