use crate::dynamic_mesh::*;
use crate::types::*;

impl DynamicMesh
{
    pub fn face_normal(&self, face_id: &FaceID) -> Vec3
    {
        let mut walker = self.walker_from_face(face_id);
        let p0 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let v0 = *self.position(&walker.vertex_id().unwrap()) - p0;
        walker.next();
        let v1 = *self.position(&walker.vertex_id().unwrap()) - p0;

        let mut dir = v0.cross(&v1);
        dir.normalize_mut();
        dir
    }

    pub fn face_area(&self, face_id: &FaceID) -> f32
    {
        let mut walker = self.walker_from_face(face_id);
        let p0 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let v0 = *self.position(&walker.vertex_id().unwrap()) - p0;
        walker.next();
        let v1 = *self.position(&walker.vertex_id().unwrap()) - p0;

        v0.cross(&v1).norm()
    }

    pub fn face_center(&self, face_id: &FaceID) -> Vec3
    {
        let mut walker = self.walker_from_face(face_id);
        let p0 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let p1 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let p2 = *self.position(&walker.vertex_id().unwrap());

        (p0 + p1 + p2)/3.0
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic_mesh::test_utility::*;

    #[test]
    fn test_face_normal() {
        let mesh = create_single_face();
        let computed_normal = mesh.face_normal(&FaceID::new(0));
        assert_eq!(0.0, computed_normal.x);
        assert_eq!(1.0, computed_normal.y);
        assert_eq!(0.0, computed_normal.z);
    }
}