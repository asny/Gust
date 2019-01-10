
use tri_mesh::prelude::*;

const MARGIN: f32 = 0.00001;
const SQR_MARGIN: f32 = MARGIN * MARGIN;

#[derive(Debug, PartialEq)]
pub struct Intersection {
    pub id1: Primitive,
    pub id2: Primitive,
    pub point: Vec3
}

pub fn find_face_edge_intersections(mesh1: &Mesh, face_id: &FaceID, mesh2: &Mesh, edge: &(VertexID, VertexID)) -> Option<(Intersection, Option<Intersection>)>
{
    let p0 = mesh2.vertex_position(&edge.0);
    let p1 = mesh2.vertex_position(&edge.1);

    if let Some(intersection) = find_face_line_piece_intersection(mesh1, face_id, &p0, &p1)
    {
        let mut id2 = Primitive::Edge(edge.clone());
        let mut intersection2 = None;
        if (intersection.point - p0).magnitude() < MARGIN {
            id2 = Primitive::Vertex(edge.0);
            if let Some(id) = find_face_point_intersection(mesh1, face_id, p1 ) {
                intersection2 = Some(Intersection{id1: id, id2: Primitive::Vertex(edge.1), point: *p1});
            }
        }
        else if (intersection.point - p1).magnitude() < MARGIN {
            id2 = Primitive::Vertex(edge.1);
        }
        return Some((Intersection {id1: intersection.id, id2, point: intersection.point}, intersection2));
    }

    None
}

#[derive(Debug, PartialEq)]
pub struct FaceLinePieceIntersection {
    pub id: Primitive,
    pub point: Vec3
}

pub fn find_face_line_piece_intersection(mesh: &Mesh, face_id: &FaceID, point0: &Vec3, point1: &Vec3) -> Option<FaceLinePieceIntersection>
{
    let p = mesh.vertex_position(&mesh.walker_from_face(face_id).vertex_id().unwrap());
    let n = mesh.face_normal(face_id);

    match plane_line_piece_intersection(&point0, &point1, p, &n) {
        Some(PlaneLinepieceIntersectionResult::LineInPlane) => {
            if let Some(id) = find_face_point_intersection_when_point_in_plane(mesh, face_id, point0 ) {
                return Some(FaceLinePieceIntersection{id, point: *point0});
            }
            if let Some(id) = find_face_point_intersection_when_point_in_plane(mesh, face_id, point1 ) {
                return Some(FaceLinePieceIntersection{id, point: *point1});
            }
        },
        Some(PlaneLinepieceIntersectionResult::P0InPlane) => {
            if let Some(id) = find_face_point_intersection_when_point_in_plane(mesh, face_id, point0 ) {
                return Some(FaceLinePieceIntersection{id, point: *point0});
            }
        },
        Some(PlaneLinepieceIntersectionResult::P1InPlane) => {
            if let Some(id) = find_face_point_intersection_when_point_in_plane(mesh, face_id, point1 ) {
                return Some(FaceLinePieceIntersection{id, point: *point1});
            }
        },
        Some(PlaneLinepieceIntersectionResult::Intersection(point)) => {
            if let Some(id) = find_face_point_intersection_when_point_in_plane(mesh, face_id, &point ) {
                return Some(FaceLinePieceIntersection{id, point});
            }
        },
        None => {}
    }

    None
}

pub fn find_edge_intersection(mesh: &Mesh, edge: &(VertexID, VertexID), point: &Vec3) -> Option<Primitive>
{
    let p0 = mesh.vertex_position(&edge.0);
    let p1 = mesh.vertex_position(&edge.1);
    if (point - p0).magnitude2() < SQR_MARGIN {
        return Some(Primitive::Vertex(edge.0));
    }
    if (point - p1).magnitude2() < SQR_MARGIN {
        return Some(Primitive::Vertex(edge.1));
    }
    if point_line_segment_distance(point, p0, p1) < MARGIN
    {
        return Some(Primitive::Edge(edge.clone()));
    }
    None
}

pub fn find_face_point_intersection(mesh: &Mesh, face_id: &FaceID, point: &Vec3) -> Option<Primitive>
{
    let p = *mesh.vertex_position(&mesh.walker_from_face(face_id).vertex_id().unwrap());
    let n = mesh.face_normal(face_id);
    let v = (point - p).normalize();
    if n.dot(v).abs() > MARGIN { return None; }

    find_face_point_intersection_when_point_in_plane(mesh, face_id, point)
}

// Assumes that the point lies in the plane spanned by the face
fn find_face_point_intersection_when_point_in_plane(mesh: &Mesh, face_id: &FaceID, point: &Vec3) -> Option<Primitive>
{
    let face_vertices = mesh.ordered_face_vertices(face_id);
    let v0 = face_vertices.0;
    let v1 = face_vertices.1;
    let v2 = face_vertices.2;

    let a = mesh.vertex_position(&v0);
    let b = mesh.vertex_position(&v1);
    let c = mesh.vertex_position(&v2);

    if point_line_segment_distance(point, a, b) < MARGIN
    {
        if (*a - *point).magnitude2() < SQR_MARGIN { return Some(Primitive::Vertex(v0)); }
        if (*b - *point).magnitude2() < SQR_MARGIN { return Some(Primitive::Vertex(v1)); }
        return Some(Primitive::Edge((v0, v1)));
    }
    if (*c - *point).magnitude2() < SQR_MARGIN { return Some(Primitive::Vertex(v2)); }

    if point_line_segment_distance(point, b, c) < MARGIN { return Some(Primitive::Edge((v1, v2))); }
    if point_line_segment_distance(point, a, c) < MARGIN { return Some(Primitive::Edge((v0, v2))); }

    // Test whether the intersection point lies inside the face
    let coords = barycentric(point, a, b, c);
    if 0.0 < coords.0 && coords.0 < 1.0 && 0.0 < coords.1 && coords.1 < 1.0 && 0.0 < coords.2 && coords.2 < 1.0
    {
        return Some(Primitive::Face(face_id.clone()));
    }
    None
}

#[derive(Debug, PartialEq)]
enum PlaneLinepieceIntersectionResult
{
    P0InPlane,
    P1InPlane,
    LineInPlane,
    Intersection(Vec3)
}

fn plane_line_piece_intersection(p0: &Vec3, p1: &Vec3, p: &Vec3, n: &Vec3) -> Option<PlaneLinepieceIntersectionResult>
{
    let ap0 = *p0 - *p;
    let ap1 = *p1 - *p;

    let d0 = n.dot(ap0);
    let d1 = n.dot(ap1);

    if d0.abs() < MARGIN && d1.abs() < MARGIN { // p0 and p1 lies in the plane
        Some(PlaneLinepieceIntersectionResult::LineInPlane)
    }
    else if d0.abs() < MARGIN { // p0 lies in the plane
        Some(PlaneLinepieceIntersectionResult::P0InPlane)
    }
    else if d1.abs() < MARGIN { // p1 lies in the plane
        Some(PlaneLinepieceIntersectionResult::P1InPlane)
    }
    else if d0.signum() != d1.signum() // The edge intersects the plane
    {
        // Find intersection point:
        let p01 = *p1 - *p0;
        let t = n.dot(-ap0) / n.dot(p01);
        let point = p0 + p01 * t;
        Some(PlaneLinepieceIntersectionResult::Intersection(point))
    }
    else {
        None
    }
}

// Compute barycentric coordinates (u, v, w) for
// point p with respect to triangle (a, b, c)
fn barycentric(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3) -> (f32, f32, f32)
{
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);
    let denom = d00 * d11 - d01 * d01;
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    (u, v, w)
}

fn point_line_segment_distance( point: &Vec3, p0: &Vec3, p1: &Vec3 ) -> f32
{
    let v  = p1 - p0;
    let w  = point - p0;

    let c1 = w.dot(v);
    if c1 <= 0.0 { return w.magnitude(); }

    let c2 = v.dot(v);
    if c2 <= c1 { return (point - p1).magnitude(); }

    let b = c1 / c2;
    let pb = p0 + b * v;
    (point - &pb).magnitude()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_face_intersection()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.0];
        let mut mesh = MeshBuilder::new().with_positions(positions).build().unwrap();
        mesh.scale(3.0);
        let face_id = mesh.face_iter().next().unwrap();
        let (v0, v1, v2) = mesh.ordered_face_vertices(&face_id);
        let p0 = mesh.vertex_position(&v0);
        let p1 = mesh.vertex_position(&v1);
        let p2 = mesh.vertex_position(&v2);

        let edge_midpoint = (p1 + p2) * 0.5;
        let face_midpoint = mesh.face_center(&face_id);

        // Vertex intersection
        let mut result = find_face_point_intersection_when_point_in_plane(&mesh, &face_id, p0);
        assert_eq!(result, Some(Primitive::Vertex(v0)));

        let dir_away_from_p0 = -(0.5 * (p1 + p2) - p0).normalize();
        result = find_face_point_intersection_when_point_in_plane(&mesh, &face_id, &(p0 + 0.99 * MARGIN * dir_away_from_p0));
        assert_eq!(result, Some(Primitive::Vertex(v0)));

        result = find_face_point_intersection_when_point_in_plane(&mesh, &face_id, &(p0 + 1.01 * MARGIN * dir_away_from_p0));
        assert_eq!(result, None);

        // Edge intersection
        result = find_face_point_intersection_when_point_in_plane(&mesh, &face_id, &edge_midpoint);
        assert_eq!(result, Some(Primitive::Edge((v1, v2))));

        let dir_away_from_edge = vec3(0.0, 1.0, 0.0);
        result = find_face_point_intersection_when_point_in_plane(&mesh, &face_id, &(edge_midpoint + 0.99 * MARGIN * dir_away_from_edge));
        assert_eq!(result, Some(Primitive::Edge((v1, v2))));

        result = find_face_point_intersection_when_point_in_plane(&mesh, &face_id, &(edge_midpoint + 1.01 * MARGIN * dir_away_from_edge));
        assert_eq!(result, None);

        // Face intersection
        result = find_face_point_intersection_when_point_in_plane(&mesh, &face_id, &face_midpoint);
        assert_eq!(result, Some(Primitive::Face(face_id)));
    }

    #[test]
    fn test_find_edge_intersection()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.0];
        let mut mesh = MeshBuilder::new().with_positions(positions).build().unwrap();
        mesh.scale(3.0);
        let edge_id = mesh.halfedge_iter().next().unwrap();
        let (v0, v1) = mesh.ordered_edge_vertices(&edge_id);
        let p0 = mesh.vertex_position(&v0);
        let p1 = mesh.vertex_position(&v1);

        // Vertex intersection
        let mut result = find_edge_intersection(&mesh, &(v0, v1), p0);
        assert_eq!(result, Some(Primitive::Vertex(v0)));

        let dir_away_from_p0 = -(p1 - p0).normalize();
        result = find_edge_intersection(&mesh, &(v0, v1), &(p0 + 0.99 * MARGIN * dir_away_from_p0));
        assert_eq!(result, Some(Primitive::Vertex(v0)));

        result = find_edge_intersection(&mesh, &(v0, v1), &(p0 + 1.01 * MARGIN * dir_away_from_p0));
        assert_eq!(result, None);

        // Edge intersection
        let edge_midpoint = (p0 + p1) * 0.5;
        result = find_edge_intersection(&mesh, &(v0, v1), &edge_midpoint);
        assert_eq!(result, Some(Primitive::Edge((v0, v1))));

        let dir_away_from_edge = dir_away_from_p0.cross(vec3(1.0, 1.0, 1.0)).normalize();
        result = find_edge_intersection(&mesh, &(v0, v1), &(edge_midpoint + 0.99 * MARGIN * dir_away_from_edge));
        assert_eq!(result, Some(Primitive::Edge((v0, v1))));

        result = find_edge_intersection(&mesh, &(v0, v1), &(edge_midpoint + 1.01 * MARGIN * dir_away_from_edge));
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_face_edge_intersection_no_intersection()
    {
        let mesh1 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]).build().unwrap();
        let mesh2 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![1.0 + MARGIN, 0.0, 0.0, 3.0, 0.0, 1.0, 4.0, 0.0, 0.0]).build().unwrap();
        let face_id = mesh1.face_iter().next().unwrap();
        let edge_id = mesh2.edge_iter().next().unwrap();

        let result = find_face_edge_intersections(&mesh1, &face_id, &mesh2, &edge_id);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_face_edge_intersection_vertex_face_intersection()
    {
        let mesh1 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]).build().unwrap();
        let mesh2 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 1.0, 0.0, 0.1, 0.0, 0.1, 1.0, 1.0, 0.0]).build().unwrap();
        let intersection_point = vec3(0.1, 0.0, 0.1);
        let face_id = mesh1.face_iter().next().unwrap();
        let edge_id = mesh2.edge_iter().find(|(v1, _)| *mesh2.vertex_position(v1) == intersection_point).unwrap();

        let result = find_face_edge_intersections(&mesh1, &face_id, &mesh2, &edge_id);
        let vertex_id = mesh2.vertex_iter().find(|v| *mesh2.vertex_position(v) == intersection_point).unwrap();
        assert_eq!(result, Some((Intersection {id1: Primitive::Face(face_id), id2: Primitive::Vertex(vertex_id), point: intersection_point}, None) ));
    }

    #[test]
    fn test_find_face_edge_intersection_edge_face_intersection()
    {
        let mesh1 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]).build().unwrap();
        let mesh2 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.1, 1.0, 0.1, 0.1, -0.1, 0.1, 1.0, 1.0, 0.0]).build().unwrap();

        let face_id = mesh1.face_iter().next().unwrap();
        let edge_id = mesh2.edge_iter().find(|(v1, v2)| mesh2.vertex_position(v1)[0] == 0.1 && mesh2.vertex_position(v2)[0] == 0.1 ).unwrap();

        let result = find_face_edge_intersections(&mesh1, &face_id, &mesh2, &edge_id);
        assert_eq!(result, Some((Intersection {id1: Primitive::Face(face_id), id2: Primitive::Edge(edge_id), point: vec3(0.1, 0.0, 0.1)}, None) ));
    }

    #[test]
    fn test_find_face_edge_intersection_two_vertex_face_intersection()
    {
        let mesh1 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]).build().unwrap();
        let mesh2 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.1, 0.0, 0.1, 0.2, 0.0, 0.2, 1.0, 1.0, 0.0]).build().unwrap();

        let face_id = mesh1.face_iter().next().unwrap();
        let edge_id = mesh2.edge_iter().find(|(v1, v2)| mesh2.vertex_position(v1)[1] == 0.0 && mesh2.vertex_position(v2)[1] == 0.0 ).unwrap();

        let result = find_face_edge_intersections(&mesh1, &face_id, &mesh2, &edge_id);
        assert_eq!(result, Some((Intersection {id1: Primitive::Face(face_id), id2: Primitive::Vertex(edge_id.0), point: vec3(0.1, 0.0, 0.1)},
            Some(Intersection {id1: Primitive::Face(face_id), id2: Primitive::Vertex(edge_id.1), point: vec3(0.2, 0.0, 0.2)})) ));
    }

    #[test]
    fn test_find_face_edge_intersection_one_vertex_face_intersection_edge_in_plane()
    {
        let mesh1 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]).build().unwrap();
        let mesh2 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.1, 0.0, 0.1, 1.2, 0.0, 0.2, 1.0, 1.0, 0.0]).build().unwrap();

        let face_id = mesh1.face_iter().next().unwrap();
        let edge_id = mesh2.edge_iter().find(|(v1, v2)| mesh2.vertex_position(v1)[1] == 0.0 && mesh2.vertex_position(v2)[1] == 0.0 ).unwrap();

        let result = find_face_edge_intersections(&mesh1, &face_id, &mesh2, &edge_id);
        assert_eq!(result, Some((Intersection {id1: Primitive::Face(face_id), id2: Primitive::Vertex(edge_id.0), point: vec3(0.1, 0.0, 0.1)}, None) ));
    }

    #[test]
    fn test_plane_line_piece_intersection_no_intersection()
    {
        let p = vec3(1.0, 1.0, 1.0);
        let n = vec3(0.0, 0.0, -1.0);

        let p0 = vec3(0.0, 0.0, 0.0);
        let p1 = vec3(0.0, 1.0, 1.0 - MARGIN);

        let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
        assert_eq!(result, None);
    }

    #[test]
    fn test_plane_line_piece_intersection_point_in_plane()
    {
        let p = vec3(1.0, 1.0, 1.0);
        let n = vec3(0.0, 0.0, -1.0);

        let p0 = vec3(0.0, 0.0, 0.0);
        let p1 = vec3(0.0, 1.0,1.0);

        let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
        assert_eq!(result, Some(PlaneLinepieceIntersectionResult::P1InPlane));
    }

    #[test]
    fn test_plane_line_piece_intersection_intersection()
    {
        let p = vec3(1.0, 1.0, 1.0);
        let n = vec3(0.0, 0.0, -1.0);

        let p0 = vec3(0.0, 1.0, 0.0);
        let p1 = vec3(0.0, 1.0,1.0 + MARGIN);

        let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
        assert_eq!(result, Some(PlaneLinepieceIntersectionResult::Intersection(vec3(0.0, 1.0, 1.0))));
    }

    #[test]
    fn test_plane_line_piece_intersection_line_in_plane()
    {
        let p = vec3(1.0, 1.0, 1.0);
        let n = vec3(0.0, 0.0, -1.0);

        let p0 = vec3(-1.0, 1.0, 1.0);
        let p1 = vec3(0.0, 1.0,1.0);

        let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
        assert_eq!(result, Some(PlaneLinepieceIntersectionResult::LineInPlane));
    }
}