//use godot::classes::fast_noise_lite::FractalType;
//use godot::classes::fast_noise_lite::NoiseType;
use godot::classes::mesh::PrimitiveType;
use godot::classes::{ArrayMesh, INode3D, MeshInstance3D, Node3D, SurfaceTool};
use godot::obj::NewAlloc;
use godot::prelude::*;

use std::time::Instant;
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};

use crate::{CUBE_TABLE, EDGE_IDX_TABLE, EDGE_TABLE, TRI_COUNT, TRI_START, TRI_TABLE};

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct MarchingCubes {
    base: Base<Node3D>,

    values: Vec<f32>,

    noise: FastNoiseLite,

    #[var]
    marching_mesh: Gd<MeshInstance3D>,

    #[export_group(name = "Generation Settings")]
    #[export]
    resolution: u8,

    #[export]
    surface_level: f32,

    #[var(set = set_refresh)]
    #[export]
    refresh: bool,

    #[export_group(name = "Noise Settings")]
    #[export]
    frequency: f32,

    #[export]
    noise_seed: i32,

    #[var(set = set_initialize_noise)]
    #[export]
    initialize_noise: bool,
}

#[godot_api]
impl INode3D for MarchingCubes {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            values: Vec::new(),
            resolution: 16,
            surface_level: 0.0,
            refresh: false,
            marching_mesh: MeshInstance3D::new_alloc(),
            frequency: 0.01,
            noise_seed: 69,
            initialize_noise: false,
            noise: FastNoiseLite::new(),
        }
    }
}

#[godot_api]
impl MarchingCubes {
    #[signal]
    fn refreshed();

    #[func]
    pub fn set_refresh(&mut self, _v: bool) {
        self.refresh = false;
        self.generate_marching_cube_mesh();
        self.signals().refreshed().emit();
    }

    #[func]
    pub fn set_initialize_noise(&mut self, _v: bool) {
        self.initialize_noise = false;
        self.initialize_noise();
    }



    fn initialize_noise(&mut self) {
        self.noise.set_seed(Some(self.noise_seed));
        self.noise.set_noise_type(Some(NoiseType::Perlin));
        self.noise.set_frequency(Some(self.frequency));
        self.noise.set_fractal_octaves(Some(12));
        self.noise.set_fractal_type(Some(FractalType::FBm));



        godot_print!("Noise initialized CUMMIES")
    }

    fn generate_noise_field(&mut self, resolution: u8) -> Vec<f32> {
        let mut field: Vec<f32> = Vec::with_capacity((resolution as usize).pow(3));

        for y in 0..resolution {
            for z in 0..resolution {
                for x in 0..resolution {
                    field.push(self.noise.get_noise_3d(x as f32, y as f32, z as f32));
                }
            }
        }

        return field;
    }

    #[func]
    pub fn get_values(&mut self) -> PackedFloat32Array {
        let mut packed_values: PackedFloat32Array = PackedFloat32Array::new();

        for &v in &self.values {
            packed_values.push(v);
        }

        return packed_values;
    }

    fn get_cube_idx(&self, cube_values: [f32; 8]) -> u8 {
        let mut cube_idx: u8 = 0;
        for (i, &cv) in cube_values.iter().enumerate() {
            if cv > self.surface_level {
                cube_idx |= 1 << i;
            }
        }
        cube_idx
    }
    fn get_verts_list(&mut self, cube_values: [f32; 8], cube_idx: u8, offset: Vector3) -> [Vector3; 12] {
        let edge_mask: u16 = EDGE_TABLE[cube_idx as usize];
        let mut verts: [Vector3; 12] = [Vector3::ZERO; 12];
    
        for i in 0..12 {
            if edge_mask & (1 << i) != 0 {
                let idx_a: usize = EDGE_IDX_TABLE[i*2] as usize;
                let idx_b: usize = EDGE_IDX_TABLE[i*2+1] as usize;

                let position_a: Vector3 = CUBE_TABLE[idx_a];
                let position_b: Vector3 = CUBE_TABLE[idx_b];

                let value_a: f32 = cube_values[idx_a];
                let value_b: f32 = cube_values[idx_b];

                if (self.surface_level-value_a).abs() < 0.00001f32 {
                    verts[i] = position_a + offset;
                    continue;
                }
                if (self.surface_level-value_b).abs() < 0.00001f32 {
                    verts[i] = position_b + offset;
                    continue;
                }
                if (value_a-value_b).abs() < 0.00001f32 {
                    verts[i] = position_a + offset;
                    continue;
                }

                let mu: f32 = (self.surface_level-value_a)/(value_b-value_a);

                verts[i] = position_a.lerp(position_b, mu) + offset;
            }
        }

        return verts
    }

    fn get_triangle_verts(&mut self, vert_list: [Vector3; 12], cube_idx: u8) -> Vec<Vector3> {
        let tri_start = TRI_START[cube_idx as usize] as usize;
        let tri_count = TRI_COUNT[cube_idx as usize] as usize;

        let mut tri_points: Vec<Vector3> = Vec::with_capacity(tri_count);

        for i in (0..tri_count).step_by(3) {
            let a = vert_list[TRI_TABLE[tri_start + i] as usize];
            let b = vert_list[TRI_TABLE[tri_start + i+1] as usize];
            let c = vert_list[TRI_TABLE[tri_start + i+2] as usize];

            tri_points.push(c);
            tri_points.push(b);
            tri_points.push(a);
        }

        return tri_points;
    }

    fn get_idx(&self, x: usize, y: usize, z: usize) -> usize {
        let idx = x + z* (self.resolution as usize) + y* (self.resolution as usize) * (self.resolution as usize);
        return idx;
    }

    fn generate_marching_cube_mesh(&mut self) {
        let start = Instant::now();
        self.values = self.generate_noise_field(self.resolution);

        let elapsed = start.elapsed();
        godot_print!("Marching Cubes took: {:?}", elapsed);
        let mut triangle_points: Vec<Vector3> = Vec::with_capacity((self.resolution as usize - 1).pow(3) * 15);

        for y in 0..(self.resolution - 1) as usize {
            for z in 0..(self.resolution - 1) as usize {
                for x in 0..(self.resolution - 1) as usize{
                    let cube_values = [
                        self.values[self.get_idx(x+1, y, z)],
                        self.values[self.get_idx(x+1, y, z+1)],
                        self.values[self.get_idx(x, y, z+1)],
                        self.values[self.get_idx(x, y, z)],
                        self.values[self.get_idx(x+1, y+1, z)],
                        self.values[self.get_idx(x+1, y+1, z+1)],
                        self.values[self.get_idx(x, y+1, z+1)],
                        self.values[self.get_idx(x, y+1, z)]
                    ];

                    let pos: Vector3 = Vector3 { x:x as f32, y: y as f32, z: z as f32 };

                    let cube_idx: u8 = self.get_cube_idx(cube_values);

                    let verts_list: [Vector3; 12] = self.get_verts_list(cube_values, cube_idx, pos);

                    let triangle_verts: Vec<Vector3> = self.get_triangle_verts(verts_list, cube_idx);

                    triangle_points.extend_from_slice(&triangle_verts);
                }
            }
        }

        let mut st: Gd<SurfaceTool> = SurfaceTool::new_gd();
        st.begin(PrimitiveType::TRIANGLES);
        for v in triangle_points {
            st.add_vertex(v);
        }

        st.generate_normals();

        let arr_mesh: Gd<ArrayMesh> = st.commit().unwrap();

        let mut marching_mesh: Gd<MeshInstance3D> = self.marching_mesh.clone();

        marching_mesh.set_mesh(&arr_mesh);

        if marching_mesh.get_parent().is_none() {
            self.base_mut().add_child(&marching_mesh);
        }


        let typ = self.noise.noise_type;
        let oct = self.noise.octaves;

        godot_print!("{:?}", typ);
        godot_print!("{:?}", oct);
    }
}
