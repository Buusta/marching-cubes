use godot::classes::fast_noise_lite::{FractalType, NoiseType};
use godot::classes::mesh::PrimitiveType;
use godot::classes::{ArrayMesh, FastNoiseLite, INode3D, MeshInstance3D, Node3D, SurfaceTool};
use godot::classes::{StaticBody3D, CollisionShape3D};
use godot::obj::NewAlloc;
use godot::prelude::*;

use std::collections::HashMap;

use std::time::Instant;

use crate::{CUBE_TABLE, EDGE_IDX_TABLE, EDGE_TABLE, TRI_COUNT, TRI_START, TRI_TABLE};

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct MarchingCubes {
    base: Base<Node3D>,

    field: Vec<f32>,

    noise: Gd<FastNoiseLite>,

    #[var]
    marching_mesh: Gd<MeshInstance3D>,

    #[export_group(name = "Generation Settings")]
    #[var(set = set_generate_planet)]
    #[export]
    generate_planet: bool,

    #[export]
    resolution: i32,

    #[export]
    surface_level: f32,

    #[export]
    planet_size: f32,

    #[export]
    planet_clip_radius: f32,

    #[export]
    noise_strength: f32,



    #[export_group(name = "Noise Settings")]
    #[export]
    noise_seed: i32,

    #[export]
    noise_type: NoiseType,

    #[export]
    frequency: f32,

    #[export]
    fractal_type: FractalType,

    #[export]
    octaves: i32,

    #[export]
    gain: f32,

    #[export]
    lacunarity: f32,
}

#[godot_api]
impl INode3D for MarchingCubes {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            field: Vec::new(),
            marching_mesh: MeshInstance3D::new_alloc(),
            noise: FastNoiseLite::new_gd(),

            generate_planet: false,
            resolution: 16,
            surface_level: 0.0,
            planet_size: 500.0,
            planet_clip_radius: 0.75,
            noise_strength: 1.0,

            noise_seed: 42,
            noise_type: NoiseType::SIMPLEX,
            frequency: 0.01,
            fractal_type: FractalType::FBM,
            octaves: 8,
            gain: 0.5,
            lacunarity: 2.0,
        }
    }
}

#[godot_api]
impl MarchingCubes {

    #[signal]
    fn refreshed();

    #[func]
    pub fn set_generate_planet(&mut self, _v: bool) {
        let start = Instant::now();
        self.generate_planet = false;
        if self.resolution > 150 {
            godot_print!("Higher resolutions may take a while to generate...")
        }
        self.initialize_noise();
        self.generate_marching_cube_mesh();
        self.signals().refreshed().emit();
        let elapsed = start.elapsed();
        godot_print!("Total Generation time: {:?}", elapsed);
    }

    #[func]
    pub fn initialize_noise(&mut self) {
        self.noise.set_seed(self.noise_seed);
        self.noise.set_noise_type(NoiseType::SIMPLEX);
        self.noise.set_frequency(self.frequency);
        self.noise.set_fractal_type(self.fractal_type);
        self.noise.set_fractal_octaves(self.octaves);
        self.noise.set_fractal_gain(self.gain);
        self.noise.set_fractal_lacunarity(self.lacunarity);

    }

    fn generate_noise_field(&mut self) -> Vec<f32> {
        let start = Instant::now();
        let mut field: Vec<f32> = Vec::with_capacity((self.resolution as usize).pow(3));

        let slices: Array<Gd<godot::classes::Image>> = self.noise.get_image_3d(self.resolution, self.resolution, self.resolution);

        for slice in slices.iter_shared() {
            let data = slice.get_data();
            let bytes: &[u8] = data.as_slice();
            for &b in bytes {
                field.push(b as f32 * (2.0 / 255.0) - 1.0);
            }
        }

        // let center = (self.resolution as f32 - 1.0) / 2.0;

        // for y in 0..self.resolution {
        //     for z in 0..self.resolution {
        //         for x in 0..self.resolution {
        //             let nx = x as f32 - center;
        //             let ny = y as f32 - center;
        //             let nz = z as f32 - center;

        //             let distance = (nx*nx + ny*ny + nz*nz).sqrt();

        //             let radius = center * self.planet_clip_radius; // roughly fits inside your cube
        //             let sphere_val = radius - distance; // positive inside sphere

        //             let noise_val = (1.0 - self.noise.get_noise_3d(x as f32, y as f32, z as f32).abs()) * self.noise_strength;

        //             field.push(sphere_val + noise_val); // combine sphere shape and noise
        //         }
        //     }
        // }

        let elapsed = start.elapsed();
        godot_print!("Generate noise field took: {:?}", elapsed);

        return field;
    }

    #[func]
    pub fn get_values(&mut self) -> PackedFloat32Array {
        let mut packed_values: PackedFloat32Array = PackedFloat32Array::new();

        for &v in &self.field {
            packed_values.push((v as f32) / 255.0 * 2.0 - 1.0);
        }

        return packed_values;
    }

    fn get_cube_idx(&self, cube_values: [f32; 8]) -> u8 {
        let mut cube_idx: u8 = 0;
        for (i, &cv) in cube_values.iter().enumerate() {
            if cv < self.surface_level {
                cube_idx |= 1 << i;
            }
        }
        cube_idx
    }
    fn get_verts_list(
        &mut self,
        cube_values: [f32; 8],
        cube_idx: u8,
        offset: Vector3,
    ) -> [Vector3; 12] {
        let edge_mask: u16 = EDGE_TABLE[cube_idx as usize];
        let mut verts: [Vector3; 12] = [Vector3::ZERO; 12];

        for i in 0..12 {
            if edge_mask & (1 << i) != 0 {
                let idx_a: usize = EDGE_IDX_TABLE[i * 2] as usize;
                let idx_b: usize = EDGE_IDX_TABLE[i * 2 + 1] as usize;

                let position_a: Vector3 = CUBE_TABLE[idx_a];
                let position_b: Vector3 = CUBE_TABLE[idx_b];

                let value_a: f32 = cube_values[idx_a];
                let value_b: f32 = cube_values[idx_b];

                let mu: f32 = (self.surface_level - value_a) / (value_b - value_a);

                verts[i] = position_a.lerp(position_b, mu) + offset;
            }
        }

        return verts;
    }

    fn get_triangle_verts(&mut self, vert_list: [Vector3; 12], cube_idx: u8) -> Vec<Vector3> {
        let tri_start = TRI_START[cube_idx as usize] as usize;
        let tri_count = TRI_COUNT[cube_idx as usize] as usize;

        let mut tri_points: Vec<Vector3> = Vec::with_capacity(tri_count);

        for i in (0..tri_count).step_by(3) {
            tri_points.push(vert_list[TRI_TABLE[tri_start + i] as usize]);
            tri_points.push(vert_list[TRI_TABLE[tri_start + i + 1] as usize]);
            tri_points.push(vert_list[TRI_TABLE[tri_start + i + 2] as usize]);
        }

        return tri_points;
    }

    fn get_idx(&self, x: usize, y: usize, z: usize) -> usize {
        let idx = x
            + z * (self.resolution as usize)
            + y * (self.resolution as usize) * (self.resolution as usize);
        return idx;
    }

    fn generate_marching_cube_mesh(&mut self) {
        self.field = self.generate_noise_field();

        let start = Instant::now();

        let mut triangle_points: Vec<Vector3> =
            Vec::with_capacity((self.resolution as usize - 1).pow(3) * 15);

        for y in 0..(self.resolution - 1) as usize {
            for z in 0..(self.resolution - 1) as usize {
                for x in 0..(self.resolution - 1) as usize {
                    let cube_values = [
                        self.field[self.get_idx(x + 1, y, z)],
                        self.field[self.get_idx(x + 1, y, z + 1)],
                        self.field[self.get_idx(x, y, z + 1)],
                        self.field[self.get_idx(x, y, z)],
                        self.field[self.get_idx(x + 1, y + 1, z)],
                        self.field[self.get_idx(x + 1, y + 1, z + 1)],
                        self.field[self.get_idx(x, y + 1, z + 1)],
                        self.field[self.get_idx(x, y + 1, z)],
                    ];

                    let pos: Vector3 = Vector3 {
                        x: x as f32,
                        y: y as f32,
                        z: z as f32,
                    };

                    let cube_idx: u8 = self.get_cube_idx(cube_values);

                    let verts_list: [Vector3; 12] = self.get_verts_list(cube_values, cube_idx, pos);

                    let triangle_verts: Vec<Vector3> =
                        self.get_triangle_verts(verts_list, cube_idx);

                    triangle_points.extend_from_slice(&triangle_verts);
                }
            }
        }

        let elapsed = start.elapsed();
        godot_print!("Marching Cubes took: {:?}", elapsed);

        let start_mesh = Instant::now();

        let mut unique_verts: Vec<Vector3> = Vec::new();
        let mut vert_map: HashMap<(i32, i32, i32), u32> = HashMap::new(); // store rounded positions to avoid float hashing
        let mut final_indices: Vec<u32> = Vec::new();



        for v in &triangle_points {
            // round to some precision to avoid float hashing issues
            let key = ((v.x * 1_000.0) as i32, (v.y * 1_000.0) as i32, (v.z * 1_000.0) as i32);

            let idx = if let Some(&existing_idx) = vert_map.get(&key) {
                existing_idx
            } else {
                let new_idx = unique_verts.len() as u32;
                unique_verts.push(*v);
                vert_map.insert(key, new_idx);
                new_idx
            };

            final_indices.push(idx);
        }


        let mut st: Gd<SurfaceTool> = SurfaceTool::new_gd();
        st.begin(PrimitiveType::TRIANGLES);


        for &idx in &final_indices {
            st.add_vertex(unique_verts[idx as usize]);
        }

        st.generate_normals();
        let arr_mesh: Gd<ArrayMesh> = st.commit().unwrap();

        let mut marching_mesh: Gd<MeshInstance3D> = self.marching_mesh.clone();

        marching_mesh.set_mesh(&arr_mesh);

        if marching_mesh.get_parent().is_none() {
            self.base_mut().add_child(&marching_mesh);
        }


        // let mut static_body: Gd<StaticBody3D> = StaticBody3D::new_alloc();
        // let mut collision_shape: Gd<CollisionShape3D> = CollisionShape3D::new_alloc();

        // // Call create_trimesh_shape on the mesh resource
        // if let Some(shape) = arr_mesh.create_trimesh_shape() {
        //     collision_shape.set_shape(&shape);
        //     static_body.add_child(&collision_shape);
        //     self.base_mut().add_child(&static_body);
        // }

        let elapsed_mesh = start_mesh.elapsed();
        godot_print!("Mesh Generation took: {:?}", elapsed_mesh);
    }
}
