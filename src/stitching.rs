
use std::collections::{HashMap, HashSet};

use tri_mesh::prelude::*;
use crate::connected_components::*;
use crate::collision::*;

#[derive(Debug)]
pub enum Error {
    EdgeToSplitDoesNotExist {message: String}
}

pub fn split_meshes_at_intersections(mesh1: &mut Mesh, mesh2: &mut Mesh) -> Result<(Vec<Mesh>, Vec<Mesh>), Error>
{
    let (components1, components2) = split_meshes_at_intersections_and_return_components(mesh1, mesh2)?;
    let mut meshes1 = Vec::new();
    for component in components1.iter() {
        meshes1.push(mesh1.clone_subset(component));
    }
    let mut meshes2 = Vec::new();
    for component in components2.iter() {
        meshes2.push(mesh2.clone_subset(component));
    }
    Ok((meshes1, meshes2))
}

pub fn split_meshes_at_intersections_and_return_components(mesh1: &mut Mesh, mesh2: &mut Mesh) -> Result<(Vec<HashSet<FaceID>>, Vec<HashSet<FaceID>>), Error>
{
    split_meshes(mesh1, mesh2)?;
    let meshes1 = split_mesh_into_components(mesh1, mesh2);
    let meshes2 = split_mesh_into_components(mesh2, mesh1);

    Ok((meshes1, meshes2))
}

fn split_mesh_into_components(mesh: &Mesh, mesh2: &Mesh) -> Vec<HashSet<FaceID>>
{
    let mut components: Vec<HashSet<FaceID>> = Vec::new();
    for face_id in mesh.face_iter() {
        if components.iter().find(|com| com.contains(&face_id)).is_none() {
            let component = connected_component_with_limit(mesh, face_id,
                                                           &|halfedge_id| { is_at_seam(mesh, mesh2, halfedge_id) });
            components.push(component);
        }
    }
    components
}

fn is_at_seam(mesh1: &Mesh, mesh2: &Mesh, halfedge_id: HalfEdgeID) -> bool
{
    let (p10, p11) = mesh1.edge_positions(halfedge_id);
    for halfedge_id2 in mesh2.edge_iter() {
        let (p20, p21) = mesh2.edge_positions(halfedge_id2);
        if point_and_point_intersects(p10, p20) && point_and_point_intersects(p11, p21) ||
            point_and_point_intersects(p11, p20) && point_and_point_intersects(p10, p21)
        {
            if mesh1.is_edge_on_boundary(halfedge_id) || mesh2.is_edge_on_boundary(halfedge_id2) {
                return true;
            }
            let mut walker1 = mesh1.walker_from_halfedge(halfedge_id);
            let mut walker2 = mesh2.walker_from_halfedge(halfedge_id2);
            let face_id10 = walker1.face_id().unwrap();
            let face_id11 = walker1.as_twin().face_id().unwrap();
            let face_id20 = walker2.face_id().unwrap();
            let face_id21 = walker2.as_twin().face_id().unwrap();
            if (!face_and_face_overlaps(mesh1, face_id10, mesh2, face_id20) &&
                !face_and_face_overlaps(mesh1, face_id10, mesh2, face_id21)) ||
                (!face_and_face_overlaps(mesh1, face_id11, mesh2, face_id20) &&
                !face_and_face_overlaps(mesh1, face_id11, mesh2, face_id21))
            {
                return true;
            }
        }
    }
    false
}

fn split_meshes(mesh1: &mut Mesh, mesh2: &mut Mesh) -> Result<HashSet<(VertexID, VertexID)>, Error>
{
    let mut intersections = find_intersections(mesh1, mesh2);
    let mut stitches = HashSet::new();
    while let Some((ref new_edges1, ref new_edges2)) = split_at_intersections(mesh1, mesh2, &intersections, &mut stitches)?
    {
        intersections = find_intersections_between_edge_face(mesh1, new_edges1, mesh2, new_edges2);
    }
    Ok(stitches)
}

fn split_at_intersections(mesh1: &mut Mesh, mesh2: &mut Mesh, intersections: &HashMap<(Primitive, Primitive), Vec3>, stitches: &mut HashSet<(VertexID, VertexID)>) -> Result<Option<(Vec<(VertexID, VertexID)>, Vec<(VertexID, VertexID)>)>, Error>
{
    let mut new_edges1 = Vec::new();
    let mut new_edges2 = Vec::new();

    // Split faces
    let mut new_intersections: HashMap<(Primitive, Primitive), Vec3> = HashMap::new();
    let mut face_splits1 = HashMap::new();
    let mut face_splits2= HashMap::new();
    for ((id1, id2), point) in intersections.iter()
    {
        if let Primitive::Face(face_id) = id1
        {
            match find_face_primitive_to_split(&face_splits1, mesh1, *face_id, point) {
                Primitive::Vertex(vertex_id) => { new_intersections.insert((Primitive::Vertex(vertex_id), *id2), *point); },
                Primitive::Edge(edge) => { new_intersections.insert((Primitive::Edge(edge), *id2), *point); },
                Primitive::Face(split_face_id) => {
                    let vertex_id = mesh1.split_face(split_face_id, point.clone());
                    insert_faces(&mut face_splits1, mesh1, *face_id, vertex_id);
                    for halfedge_id in mesh1.vertex_halfedge_iter(vertex_id) {
                        new_edges1.push(mesh1.ordered_edge_vertices(halfedge_id));
                    }
                    new_intersections.insert((Primitive::Vertex(vertex_id), *id2), *point);
                }
            }
        }
        else if let Primitive::Face(face_id) = id2
        {
            match find_face_primitive_to_split(&face_splits2, mesh2, *face_id, point) {
                Primitive::Vertex(vertex_id) => { new_intersections.insert((*id1, Primitive::Vertex(vertex_id)), *point); },
                Primitive::Edge(edge) => { new_intersections.insert((*id1, Primitive::Edge(edge)), *point); },
                Primitive::Face(split_face_id) => {
                    let vertex_id = mesh2.split_face(split_face_id, point.clone());
                    insert_faces(&mut face_splits2, mesh2, *face_id, vertex_id);
                    for halfedge_id in mesh2.vertex_halfedge_iter(vertex_id) {
                        new_edges2.push(mesh2.ordered_edge_vertices(halfedge_id));
                    }
                    new_intersections.insert((*id1, Primitive::Vertex(vertex_id)), *point);
                }
            }
        }
        else {
            new_intersections.insert((*id1, *id2), *point);
        }
    }

    // Split edges
    let mut edge_splits1 = HashMap::new();
    let mut edge_splits2 = HashMap::new();
    for ((id1, id2), point) in new_intersections.drain()
    {
        let vertex_id1 = match id1 {
            Primitive::Vertex(vertex_id) => { vertex_id },
            Primitive::Edge(edge) => {
                match find_edge_primitive_to_split(&edge_splits1, mesh1, edge, &point) {
                    Primitive::Vertex(vertex_id) => { vertex_id },
                    Primitive::Edge(split_edge) => {
                        let halfedge_id = mesh1.connecting_edge(split_edge.0, split_edge.1).ok_or(
                            Error::EdgeToSplitDoesNotExist {message: format!("Cannot find edge ({}, {})", split_edge.0, split_edge.1)}
                        )?;
                        let vertex_id = mesh1.split_edge(halfedge_id, point);
                        insert_edges(&mut edge_splits1, edge, split_edge, vertex_id);
                        for halfedge_id in mesh1.vertex_halfedge_iter(vertex_id) {
                            let vid = mesh1.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                            if vid != split_edge.0 && vid != split_edge.1
                            {
                                new_edges1.push(mesh1.ordered_edge_vertices(halfedge_id));
                            }
                        }
                        vertex_id
                    },
                    _ => {unreachable!()}
                }
            },
            _ => {unreachable!()}
        };
        let vertex_id2 = match id2 {
            Primitive::Vertex(vertex_id) => { vertex_id },
            Primitive::Edge(edge) => {
                match find_edge_primitive_to_split(&edge_splits2, mesh2, edge, &point) {
                    Primitive::Vertex(vertex_id) => { vertex_id },
                    Primitive::Edge(split_edge) => {
                        let halfedge_id = mesh2.connecting_edge(split_edge.0, split_edge.1).ok_or(
                            Error::EdgeToSplitDoesNotExist {message: format!("Cannot find edge ({}, {})", split_edge.0, split_edge.1)}
                        )?;
                        let vertex_id = mesh2.split_edge(halfedge_id, point);
                        insert_edges(&mut edge_splits2, edge, split_edge, vertex_id);
                        for halfedge_id in mesh2.vertex_halfedge_iter(vertex_id) {
                            let vid = mesh2.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                            if vid != split_edge.0 && vid != split_edge.1
                            {
                                new_edges2.push(mesh2.ordered_edge_vertices(halfedge_id));
                            }
                        }
                        vertex_id
                    },
                    _ => {unreachable!()}
                }
            },
            _ => {unreachable!()}
        };

        stitches.insert((vertex_id1, vertex_id2));
    }
    if new_edges1.len() > 0 && new_edges2.len() > 0 { Ok(Some((new_edges1, new_edges2))) }
    else {Ok(None)}
}

fn find_face_primitive_to_split(face_splits: &HashMap<FaceID, HashSet<FaceID>>, mesh: &Mesh, face_id: FaceID, point: &Vec3) -> Primitive
{
    if let Some(new_faces) = face_splits.get(&face_id)
    {
        for new_face_id in new_faces
        {
            if let Some(id) = find_face_point_intersection(mesh, *new_face_id, point) { return id; }
        }
        unreachable!()
    }
    Primitive::Face(face_id)
}

fn find_edge_primitive_to_split(edge_splits: &HashMap<(VertexID, VertexID), HashSet<(VertexID, VertexID)>>, mesh: &Mesh, edge: (VertexID, VertexID), point: &Vec3) -> Primitive
{
    if let Some(new_edges) = edge_splits.get(&edge)
    {
        for new_edge in new_edges
        {
            if let Some(id) = find_edge_intersection(mesh, *new_edge, point) { return id; }
        }
        unreachable!()
    }
    Primitive::Edge(edge)
}

fn insert_edges(edge_list: &mut HashMap<(VertexID, VertexID), HashSet<(VertexID, VertexID)>>, edge: (VertexID, VertexID), split_edge: (VertexID, VertexID), vertex_id: VertexID)
{
    if !edge_list.contains_key(&edge) { edge_list.insert(edge, HashSet::new()); }
    let list = edge_list.get_mut(&edge).unwrap();
    list.remove(&split_edge);
    list.insert((split_edge.0, vertex_id));
    list.insert((split_edge.1, vertex_id));
}

fn insert_faces(face_list: &mut HashMap<FaceID, HashSet<FaceID>>, mesh: &Mesh, face_id: FaceID, vertex_id: VertexID)
{
    if !face_list.contains_key(&face_id) { face_list.insert(face_id, HashSet::new()); }
    let list = face_list.get_mut(&face_id).unwrap();

    let mut iter = mesh.vertex_halfedge_iter(vertex_id);
    list.insert(mesh.walker_from_halfedge(iter.next().unwrap()).face_id().unwrap());
    list.insert(mesh.walker_from_halfedge(iter.next().unwrap()).face_id().unwrap());
    list.insert(mesh.walker_from_halfedge(iter.next().unwrap()).face_id().unwrap());
}

fn find_intersections(mesh1: &Mesh, mesh2: &Mesh) -> HashMap<(Primitive, Primitive), Vec3>
{
    let edges1 = mesh1.edge_iter().map(|halfedge_id| mesh1.ordered_edge_vertices(halfedge_id)).collect();
    let edges2 = mesh2.edge_iter().map(|halfedge_id| mesh2.ordered_edge_vertices(halfedge_id)).collect();
    find_intersections_between_edge_face(mesh1, &edges1, mesh2, &edges2)
}

fn find_intersections_between_edge_face(mesh1: &Mesh, edges1: &Vec<(VertexID, VertexID)>, mesh2: &Mesh, edges2: &Vec<(VertexID, VertexID)>) -> HashMap<(Primitive, Primitive), Vec3>
{
    let mut intersections: HashMap<(Primitive, Primitive), Vec3> = HashMap::new();
    for edge1 in edges1
    {
        for face_id2 in mesh2.face_iter()
        {
            if let Some(result) = find_face_edge_intersections(mesh2, face_id2, mesh1,*edge1)
            {
                let intersection = result.0;
                intersections.insert((intersection.id2, intersection.id1), intersection.point);
                if let Some(other_intersection) = result.1
                {
                    intersections.insert((other_intersection.id2, other_intersection.id1), other_intersection.point);
                }
            }
        }
    }
    for edge2 in edges2
    {
        for face_id1 in mesh1.face_iter()
        {
            if let Some(result) = find_face_edge_intersections(mesh1, face_id1, mesh2, *edge2)
            {
                let intersection = result.0;
                intersections.insert((intersection.id1, intersection.id2), intersection.point);
                if let Some(other_intersection) = result.1
                {
                    intersections.insert((other_intersection.id1, other_intersection.id2), other_intersection.point);
                }
            }
        }
    }
    intersections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_edge_edge_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let mesh2 = create_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 5);

        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 0.25));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 0.75));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 1.25));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 1.75));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 2.25));
    }

    #[test]
    fn test_finding_face_edge_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, -0.5, 0.0,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 2);
    }

    #[test]
    fn test_finding_face_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, 0.0, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_finding_edge_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, 0.0, 0.25,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_finding_vertex_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![1.0, 0.0, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_split_edges()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);
        let mut stitches = HashSet::new();
        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap().unwrap();

        assert_eq!(mesh1.no_vertices(), 11);
        assert_eq!(mesh1.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh1.no_faces(), 12);

        assert_eq!(mesh2.no_vertices(), 11);
        assert_eq!(mesh2.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh2.no_faces(), 12);

        assert_eq!(stitches.len(), 5);
        assert_eq!(new_edges1.len(), 8);
        assert_eq!(new_edges2.len(), 8);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_split_faces()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_shifted_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 8);

        let mut stitches = HashSet::new();
        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap().unwrap();

        assert_eq!(mesh1.no_vertices(), 14);
        assert_eq!(mesh1.no_faces(), 19);
        assert_eq!(mesh1.no_halfedges(), 19 * 3 + 7);

        assert_eq!(mesh2.no_vertices(), 14);
        assert_eq!(mesh2.no_faces(), 19);
        assert_eq!(mesh2.no_halfedges(), 19 * 3 + 7);

        assert_eq!(stitches.len(), 8);
        assert_eq!(new_edges1.len(), 19);
        assert_eq!(new_edges2.len(), 19);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

    }

    #[test]
    fn test_split_face_two_times()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_positions(positions1).with_indices(indices1).build().unwrap();
        let area1 = mesh1.face_area(mesh1.face_iter().next().unwrap());

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.2, -0.2, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new().with_positions(positions2).with_indices(indices2).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 2);

        let mut stitches = HashSet::new();
        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap().unwrap();

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh1.no_faces(), 5);
        assert_eq!(mesh1.no_halfedges(), 5 * 3 + 3);

        let mut area_test1 = 0.0;
        for face_id in mesh1.face_iter() {
            area_test1 = area_test1 + mesh1.face_area(face_id);
        }
        assert!((area1 - area_test1).abs() < 0.001);

        assert_eq!(mesh2.no_vertices(), 5);
        assert_eq!(mesh2.no_faces(), 3);
        assert_eq!(mesh2.no_halfedges(), 3 * 3 + 5);

        assert_eq!(stitches.len(), 2);
        assert_eq!(new_edges1.len(), 6);
        assert_eq!(new_edges2.len(), 2);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_split_edge_two_times()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_positions(positions1).with_indices(indices1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.0, -0.2, 0.5,  0.0, -0.2, 1.5,  0.0, 1.5, 0.0];
        let mut mesh2 = MeshBuilder::new().with_positions(positions2).with_indices(indices2).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 2);

        let mut stitches = HashSet::new();
        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap().unwrap();

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh1.no_faces(), 3);
        assert_eq!(mesh1.no_halfedges(), 3 * 3 + 5);

        assert_eq!(mesh2.no_vertices(), 5);
        assert_eq!(mesh2.no_faces(), 3);
        assert_eq!(mesh2.no_halfedges(), 3 * 3 + 5);

        assert_eq!(stitches.len(), 2);
        assert_eq!(new_edges1.len(), 2);
        assert_eq!(new_edges2.len(), 2);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_face_face_splitting()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_positions(positions1).with_indices(indices1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.2, -0.2, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new().with_positions(positions2).with_indices(indices2).build().unwrap();

        let stitches = split_meshes(&mut mesh1, &mut mesh2).unwrap();

        assert_eq!(stitches.len(), 2);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_simple_simple_splitting()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_shifted_simple_mesh_y_z();

        let stitches = split_meshes(&mut mesh1, &mut mesh2).unwrap();

        assert_eq!(stitches.len(), 8);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_box_box_splitting()
    {
        let mut mesh1 = MeshBuilder::new().cube().build().unwrap();
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        for vertex_id in mesh2.vertex_iter() {
            mesh2.move_vertex_by(vertex_id, vec3(0.5, 0.5, 0.5));
        }
        split_meshes(&mut mesh1, &mut mesh2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_face_face_stitching_at_edge()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_positions(positions1).with_indices(indices1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0,  -2.0, 0.0, -2.0,  -2.0, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new().with_positions(positions2).with_indices(indices2).build().unwrap();

        let (meshes1, meshes2) = split_meshes_at_intersections(&mut mesh1, &mut mesh2).unwrap();
        assert_eq!(meshes1.len(), 1);
        assert_eq!(meshes2.len(), 1);

        let mut m1 = meshes1[0].clone();
        let m2 = meshes2[0].clone();
        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        assert_eq!(m1.no_faces(), 2);
        assert_eq!(m1.no_vertices(), 4);

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_face_face_stitching_at_mid_edge()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_positions(positions1).with_indices(indices1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 1.0,  -2.0, 0.0, -1.0,  -2.0, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new().with_positions(positions2).with_indices(indices2).build().unwrap();

        let (meshes1, meshes2) = split_meshes_at_intersections(&mut mesh1, &mut mesh2).unwrap();
        assert_eq!(meshes1.len(), 1);
        assert_eq!(meshes2.len(), 1);

        let mut m1 = meshes1[0].clone();
        let m2 = meshes2[0].clone();
        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        assert_eq!(m1.no_faces(), 4);
        assert_eq!(m1.no_vertices(), 6);

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_box_box_stitching()
    {
        let mut mesh1 = MeshBuilder::new().cube().build().unwrap();
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        mesh2.translate(vec3(0.5, 0.5, 0.5));

        let (meshes1, meshes2) = split_meshes_at_intersections(&mut mesh1, &mut mesh2).unwrap();
        assert_eq!(meshes1.len(), 2);
        assert_eq!(meshes2.len(), 2);

        let mut m1 = if meshes1[0].no_faces() > meshes1[1].no_faces() { meshes1[0].clone() } else { meshes1[1].clone() };
        let m2 = if meshes2[0].no_faces() > meshes2[1].no_faces() { meshes2[0].clone() } else { meshes2[1].clone() };

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();

        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_sphere_box_stitching()
    {
        let mut mesh1 = MeshBuilder::new().icosahedron().build().unwrap();
        for _ in 0..1 {
            for face_id in mesh1.face_iter() {
                let p = mesh1.face_center(face_id).normalize();
                mesh1.split_face(face_id, p);
            }
            mesh1.smooth_vertices(1.0);
            for vertex_id in mesh1.vertex_iter() {
                let p = mesh1.vertex_position(vertex_id).normalize();
                mesh1.move_vertex_to(vertex_id, p)
            }
            mesh1.flip_edges(0.5);
        }
        mesh1.translate(vec3(0.0, 1.5, 0.0));
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        mesh2.translate(vec3(0.5, 2.0, 0.5));

        let (meshes1, meshes2) = split_meshes_at_intersections(&mut mesh1, &mut mesh2).unwrap();
        assert_eq!(meshes1.len(), 2);
        assert_eq!(meshes2.len(), 2);

        let mut m1 = if meshes1[0].no_faces() > meshes1[1].no_faces() { meshes1[0].clone() } else { meshes1[1].clone() };
        let m2 = if meshes2[0].no_faces() > meshes2[1].no_faces() { meshes2[0].clone() } else { meshes2[1].clone() };

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();

        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_split_mesh_into_components()
    {
        let mesh1 = MeshBuilder::new().cube().build().unwrap();
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        mesh2.translate(vec3(0.0, 2.0, 0.0));

        let result = split_mesh_into_components(&mesh1, &mesh2);

        assert_eq!(result.len(), 2);
        assert!(result.iter().find(|cc| cc.len() == 2).is_some());
        assert!(result.iter().find(|cc| cc.len() == 10).is_some());
    }

    #[test]
    fn test_split_mesh_into_components2()
    {
        let mesh1 = MeshBuilder::new().cube().build().unwrap();

        let positions = vec![-1.0, 1.0, 1.0,  -1.0, -1.0, 1.0,  1.0, -1.0, -1.0,  1.0, 1.0, -1.0, 0.0, 2.0, 0.0 ];
        let indices = vec![0, 1, 2,  0, 2, 3,  0, 3, 4];
        let mut mesh2 = MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap();

        let result = split_mesh_into_components(&mesh2, &mesh1);

        assert_eq!(result.len(), 2);
        assert!(result.iter().find(|cc| cc.len() == 1).is_some());
        assert!(result.iter().find(|cc| cc.len() == 2).is_some());

    }

    fn create_simple_mesh_x_z() -> Mesh
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5,  0.0, 0.0, 2.0,  1.0, 0.0, 2.5];
        MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap()
    }

    fn create_simple_mesh_y_z() -> Mesh
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.5, -0.5, 0.0,  0.5, -0.5, 1.0,  0.5, 0.5, 0.5,  0.5, 0.5, 1.5,  0.5, -0.5, 2.0,  0.5, 0.5, 2.5];
        MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap()
    }

    fn create_shifted_simple_mesh_y_z() -> Mesh
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.5, -0.5, -0.2,  0.5, -0.5, 0.8,  0.5, 0.5, 0.3,  0.5, 0.5, 1.3,  0.5, -0.5, 1.8,  0.5, 0.5, 2.3];
        MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap()
    }
}