use std::rc::{Rc};
use crate::mesh::Mesh;
use crate::mesh::ids::*;
use crate::mesh::connectivity_info::{HalfEdge, ConnectivityInfo};
use std::collections::HashSet;
use std::iter::FromIterator;

pub type VertexIter = Box<Iterator<Item = VertexID>>;
pub type HalfEdgeIter = Box<Iterator<Item = HalfEdgeID>>;
pub type FaceIter = Box<Iterator<Item = FaceID>>;
pub type HalfEdgeTwinsIter = Box<Iterator<Item = (HalfEdgeID, HalfEdgeID)>>;
pub type EdgeIter = Box<Iterator<Item = (VertexID, VertexID)>>;

impl Mesh
{
    pub fn walker(&self) -> Walker
    {
        Walker::new(&self.connectivity_info)
    }

    pub fn walker_from_vertex(&self, vertex_id: &VertexID) -> Walker
    {
        Walker::new(&self.connectivity_info).into_vertex_halfedge_walker(vertex_id)
    }

    pub fn walker_from_halfedge(&self, halfedge_id: &HalfEdgeID) -> Walker
    {
        Walker::new(&self.connectivity_info).into_halfedge_walker(halfedge_id)
    }

    pub fn walker_from_face(&self, face_id: &FaceID) -> Walker
    {
        Walker::new(&self.connectivity_info).into_face_halfedge_walker(face_id)
    }

    pub fn vertex_halfedge_iter(&self, vertex_id: &VertexID) -> VertexHalfedgeIter
    {
        VertexHalfedgeIter::new(vertex_id, &self.connectivity_info)
    }

    pub fn face_halfedge_iter(&self, face_id: &FaceID) -> FaceHalfedgeIter
    {
        FaceHalfedgeIter::new(face_id, &self.connectivity_info)
    }

    pub fn vertex_iter(&self) -> VertexIter
    {
        self.connectivity_info.vertex_iterator()
    }

    pub fn halfedge_iter(&self) -> HalfEdgeIter
    {
        self.connectivity_info.halfedge_iterator()
    }

    pub fn halfedge_twins_iter(&self) -> HalfEdgeTwinsIter
    {
        let mut values = Vec::with_capacity(self.no_halfedges()/2);
        for halfedge_id in self.halfedge_iter() {
            let twin_id = self.walker_from_halfedge(&halfedge_id).twin_id().unwrap();
            if halfedge_id < twin_id {
                values.push((halfedge_id, twin_id))
            }
        }
        Box::new(values.into_iter())
    }

    pub fn face_iter(&self) -> FaceIter
    {
        self.connectivity_info.face_iterator()
    }

    pub fn edge_iter(&self) -> EdgeIter
    {
        let set: HashSet<(VertexID, VertexID)> = HashSet::from_iter(self.halfedge_iter().map(|halfedge_id| self.ordered_edge_vertices(&halfedge_id)));
        Box::new(set.into_iter())
    }
}

pub struct VertexHalfedgeIter
{
    current: Walker,
    start: HalfEdgeID,
    is_done: bool
}

impl VertexHalfedgeIter {
    pub(crate) fn new(vertex_id: &VertexID, connectivity_info: &Rc<ConnectivityInfo>) -> VertexHalfedgeIter
    {
        let current = Walker::new(connectivity_info).into_vertex_halfedge_walker(vertex_id);
        let start = current.halfedge_id().unwrap();
        VertexHalfedgeIter { current, start, is_done: false }
    }
}

impl Iterator for VertexHalfedgeIter {
    type Item = Walker;

    fn next(&mut self) -> Option<Walker>
    {
        if self.is_done { return None; }
        let curr = self.current.clone();

        match self.current.face_id() {
            Some(_) => {
                self.current.as_previous().as_twin();
            },
            None => { // In the case there are holes in the one-ring
                self.current.as_twin();
                while let Some(_) = self.current.face_id() {
                    self.current.as_next().as_twin();
                }
                self.current.as_twin();
            }
        }
        self.is_done = self.current.halfedge_id().unwrap() == self.start;
        Some(curr)
    }
}

pub struct FaceHalfedgeIter
{
    current: Walker,
    start: HalfEdgeID,
    is_done: bool
}

impl FaceHalfedgeIter {
    pub(crate) fn new(face_id: &FaceID, connectivity_info: &Rc<ConnectivityInfo>) -> FaceHalfedgeIter
    {
        let current = Walker::new(connectivity_info).into_face_halfedge_walker(face_id);
        let start = current.halfedge_id().unwrap();
        FaceHalfedgeIter { current, start, is_done: false }
    }
}

impl Iterator for FaceHalfedgeIter {
    type Item = Walker;

    fn next(&mut self) -> Option<Walker>
    {
        if self.is_done { return None; }
        let curr = self.current.clone();
        self.current.as_next();
        self.is_done = self.current.halfedge_id().unwrap() == self.start;
        Some(curr)
    }
}

#[derive(Clone, Debug)]
pub struct Walker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: Option<HalfEdgeID>,
    current_info: Option<HalfEdge>
}

impl Walker
{
    pub(crate) fn new(connectivity_info: &Rc<ConnectivityInfo>) -> Self
    {
        Walker {current: None, current_info: None, connectivity_info: connectivity_info.clone()}
    }

    pub fn into_vertex_halfedge_walker(mut self, vertex_id: &VertexID) -> Self
    {
        self.as_vertex_halfedge_walker(vertex_id);
        self
    }

    pub fn as_vertex_halfedge_walker(&mut self, vertex_id: &VertexID) -> &mut Self
    {
        let halfedge_id = self.connectivity_info.vertex_halfedge(vertex_id);
        self.set_current(halfedge_id);
        self
    }

    pub fn into_halfedge_walker(mut self, halfedge_id: &HalfEdgeID) -> Self
    {
        self.as_halfedge_walker(halfedge_id);
        self
    }

    pub fn as_halfedge_walker(&mut self, halfedge_id: &HalfEdgeID) -> &mut Self
    {
        let halfedge_id = Some(halfedge_id.clone());
        self.set_current(halfedge_id);
        self
    }

    pub fn into_face_halfedge_walker(mut self, face_id: &FaceID) -> Self
    {
        self.as_face_halfedge_walker(face_id);
        self
    }

    pub fn as_face_halfedge_walker(&mut self, face_id: &FaceID) -> &mut Self
    {
        let halfedge_id = self.connectivity_info.face_halfedge(face_id);
        self.set_current(halfedge_id);
        self
    }

    pub fn into_twin(mut self) -> Self
    {
        self.as_twin();
        self
    }

    pub fn as_twin(&mut self) -> &mut Self
    {
        let halfedge_id = match self.current_info {
            Some(ref current_info) => { current_info.twin.clone() },
            None => None
        };
        self.set_current(halfedge_id);
        self
    }

    pub fn twin_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.twin.clone() }
        else { None }
    }

    pub fn into_next(mut self) -> Self
    {
        self.as_next();
        self
    }

    pub fn as_next(&mut self) -> &mut Self
    {
        let halfedge_id = match self.current_info {
            Some(ref current_info) => { current_info.next.clone() },
            None => None
        };
        self.set_current(halfedge_id);
        self
    }

    pub fn next_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.next.clone() }
        else { None }
    }

    pub fn as_previous(&mut self) -> &mut Self
    {
        self.as_next().as_next()
    }

    pub fn into_previous(mut self) -> Self
    {
        self.as_next().as_next();
        self
    }

    pub fn previous_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(ref next_id) = self.next_id() { Walker::new(&self.connectivity_info.clone()).into_halfedge_walker(next_id).next_id() }
        else { None }
    }

    pub fn vertex_id(&self) -> Option<VertexID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.vertex.clone() }
        else { None }
    }

    pub fn halfedge_id(&self) -> Option<HalfEdgeID>
    {
        self.current.clone()
    }

    pub fn face_id(&self) -> Option<FaceID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.face.clone() }
        else { None }
    }

    fn set_current(&mut self, halfedge_id: Option<HalfEdgeID>)
    {
        self.current_info = if let Some(ref id) = halfedge_id { self.connectivity_info.halfedge(id) } else { None };
        self.current = halfedge_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::test_utility::*;

    #[test]
    fn test_vertex_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for _ in mesh.vertex_iter() {
            i = i+1;
        }
        assert_eq!(4, i);

        // Test that two iterations return the same result
        let vec: Vec<VertexID> = mesh.vertex_iter().collect();
        i = 0;
        for vertex_id in mesh.vertex_iter() {
            assert_eq!(vertex_id, vec[i]);
            i = i+1;
        }
    }

    #[test]
    fn test_halfedge_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for _ in mesh.halfedge_iter() {
            i = i+1;
        }
        assert_eq!(12, i);

        // Test that two iterations return the same result
        let vec: Vec<HalfEdgeID> = mesh.halfedge_iter().collect();
        i = 0;
        for halfedge_id in mesh.halfedge_iter() {
            assert_eq!(halfedge_id, vec[i]);
            i = i+1;
        }
    }

    #[test]
    fn test_face_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for _ in mesh.face_iter() {
            i = i+1;
        }
        assert_eq!(3, i);

        // Test that two iterations return the same result
        let vec: Vec<FaceID> = mesh.face_iter().collect();
        i = 0;
        for face_id in mesh.face_iter() {
            assert_eq!(face_id, vec[i]);
            i = i+1;
        }
    }

    #[test]
    fn test_vertex_halfedge_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        let vertex_id = mesh.vertex_iter().last().unwrap();
        for edge in mesh.vertex_halfedge_iter(&vertex_id) {
            assert!(edge.vertex_id().is_some());
            i = i + 1;
        }
        assert_eq!(i, 3, "All edges of a one-ring are not visited");
    }

    #[test]
    fn test_vertex_halfedge_iterator_with_holes() {
        let indices: Vec<u32> = vec![0, 2, 3,  0, 4, 1,  0, 1, 2];
        let positions: Vec<f32> = vec![0.0; 5 * 3];
        let mesh = Mesh::new_with_connectivity(indices, positions, None);

        let mut i = 0;
        for edge in mesh.vertex_halfedge_iter(&VertexID::new(0)) {
            assert!(edge.vertex_id().is_some());
            i = i+1;
        }
        assert_eq!(i,4, "All edges of a one-ring are not visited");

    }

    #[test]
    fn test_face_halfedge_iterator() {
        let mesh = create_single_face();
        let mut i = 0;
        for edge in mesh.face_halfedge_iter(&FaceID::new(0)) {
            assert!(edge.halfedge_id().is_some());
            assert!(edge.face_id().is_some());
            i = i+1;
        }
        assert_eq!(i, 3, "All edges of a face are not visited");
    }
}