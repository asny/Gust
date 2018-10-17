
use dynamic_mesh::*;

#[derive(Debug)]
pub enum Error {
    CannotFlipEdgeOnBoundary {message: String}
}

impl DynamicMesh
{
    pub fn flip_edge(&mut self, halfedge_id: &HalfEdgeID) -> Result<(), Error>
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let face_id = walker.face_id().ok_or(Error::CannotFlipEdgeOnBoundary {message: format!("Trying to flip edge on boundary")})?;
        let next_id = walker.next_id().unwrap();
        let previous_id = walker.previous_id().unwrap();
        let v0 = walker.vertex_id().unwrap();

        walker.twin();
        let twin_id = walker.halfedge_id().unwrap();
        let twin_face_id = walker.face_id().ok_or(Error::CannotFlipEdgeOnBoundary {message: format!("Trying to flip edge on boundary")})?;
        let twin_next_id = walker.next_id().unwrap();
        let twin_previous_id = walker.previous_id().unwrap();
        let v1 = walker.vertex_id().unwrap();

        let v2 = walker.next().vertex_id().unwrap();
        let v3 = walker.previous().twin().next().vertex_id().unwrap();

        self.connectivity_info.set_face_halfedge(&face_id, previous_id);
        self.connectivity_info.set_face_halfedge(&twin_face_id, twin_previous_id);

        self.connectivity_info.set_vertex_halfedge(&v0, next_id);
        self.connectivity_info.set_vertex_halfedge(&v1, twin_next_id);

        self.connectivity_info.set_halfedge_next(&halfedge_id, previous_id);
        self.connectivity_info.set_halfedge_next(&next_id, twin_id);
        self.connectivity_info.set_halfedge_next(&previous_id, twin_next_id);
        self.connectivity_info.set_halfedge_next(&twin_id, twin_previous_id);
        self.connectivity_info.set_halfedge_next(&twin_next_id, halfedge_id.clone());
        self.connectivity_info.set_halfedge_next(&twin_previous_id, next_id);

        self.connectivity_info.set_halfedge_vertex(&halfedge_id, v3);
        self.connectivity_info.set_halfedge_vertex(&twin_id, v2);

        self.connectivity_info.set_halfedge_face(&next_id, twin_face_id);
        self.connectivity_info.set_halfedge_face(&twin_next_id, face_id);

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_flip_edge()
    {
        let mut mesh = ::models::create_plane().unwrap().to_dynamic();
        for halfedge_id in mesh.halfedge_iterator() {
            if !mesh.on_boundary(&halfedge_id) {
                let (v0, v1) = mesh.edge_vertices(&halfedge_id);
                mesh.flip_edge(&halfedge_id).unwrap();
                let (v2, v3) = mesh.edge_vertices(&halfedge_id);
                assert_ne!(v0, v2);
                assert_ne!(v1, v2);
                assert_ne!(v0, v3);
                assert_ne!(v1, v3);

                mesh.test_is_valid().unwrap();
            }
        }
    }
}