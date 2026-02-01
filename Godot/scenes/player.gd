extends RigidBody3D

@export var gravity_object: Node3D
@onready var collision_shape: CollisionShape3D = $CollisionShape3D

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass # Replace with function body.


func _physics_process(delta: float) -> void:
	var gravity_dir: Vector3 = (gravity_object.global_position - global_transform.origin).normalized()

	# Apply gravity force
	var force = mass * 9.81
	apply_central_force(force * gravity_dir)

	# Keep forward direction
	var forward = -global_transform.basis.z
	var right = forward.cross(gravity_dir).normalized()
	forward = gravity_dir.cross(right).normalized()

	# Construct a new Basis so -Y points to gravity_dir
	var new_basis = Basis()
	new_basis.y = -gravity_dir
	new_basis.x = right
	new_basis.z = -forward

	# Smoothly interpolate rotation
	global_transform.basis = global_transform.basis.slerp(new_basis, delta * 5.0)
