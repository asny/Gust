use crate::prelude::*;
use crate::connected_components::*;
use std::collections::HashSet;

impl Mesh
{
    pub fn split(&self, is_at_split: &Fn(&Mesh, &HalfEdgeID) -> bool) -> Vec<Mesh>
    {
        let mut components: Vec<HashSet<FaceID>> = Vec::new();
        for face_id in self.face_iter() {
            if components.iter().find(|com| com.contains(&face_id)).is_none() {
                components.push(connected_component_with_limit(self, &face_id, &|halfedge_id| is_at_split(self, &halfedge_id)));
            }
        }

        let mut meshes = Vec::new();
        for component in components {
            meshes.push(self.clone_subset(&component));
        }

        meshes
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utility::*;
    use crate::MeshBuilder;

    #[test]
    fn test_splitting()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5,  0.0, 0.0, 2.0,  1.0, 0.0, 2.5];
        let mesh = MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap();

        let meshes = mesh.split(&|mesh,
            he_id| {
                let (p0, p1) = mesh.edge_positions(he_id);
                p0.z > 0.75 && p0.z < 1.75 && p1.z > 0.75 && p1.z < 1.75
            });

        assert_eq!(meshes.len(), 2);
        let m1 = &meshes[0];
        let m2 = &meshes[1];

        test_is_valid(&mesh).unwrap();
        test_is_valid(m1).unwrap();
        test_is_valid(m2).unwrap();

        assert_eq!(m1.no_faces(), 2);
        assert_eq!(m2.no_faces(), 2);
    }
}