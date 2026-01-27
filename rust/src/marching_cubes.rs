use godot::classes::mesh::PrimitiveType;
use godot::classes::{ArrayMesh, INode3D, MeshInstance3D, Node3D, SurfaceTool};
use godot::obj::NewAlloc;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct MarchingCubes {
    base: Base<Node3D>,
    
    #[export_group(name="Generation Settings")]
    #[export]
    resolution: i32,

    #[export]
    surface_level: f32,

    #[var(set = set_refresh)]
    #[export]
    refresh: bool,

}

#[godot_api]
impl INode3D for MarchingCubes {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            resolution: 16,
            surface_level: 0.0,
            refresh: false
        }
    }
}

#[godot_api]
impl MarchingCubes {
    #[func]
    pub fn set_refresh(&mut self,_v: bool) {
        self.refresh = false;
        self.generate();
    }

    fn generate(&mut self) {

        const VERTS: [Vector3; 3] = [
        Vector3::new(1.0, 0.0, 0.0), 
        Vector3::new(0.0, 0.0, 1.0), 
        Vector3::new(0.0, 0.0, 0.0)];

        let mut st: Gd<SurfaceTool> = SurfaceTool::new_gd();
        st.begin(PrimitiveType::TRIANGLES);
        for v in VERTS {
            st.add_vertex(v);
        }

        st.generate_normals();

        let mut m: Gd<MeshInstance3D> = MeshInstance3D::new_alloc();
        let arr_mesh: Gd<ArrayMesh> = st.commit().unwrap();
        
        m.set_mesh(&arr_mesh);

        self.base_mut().add_child(&m);

        godot_print!("TEST")
    }
}