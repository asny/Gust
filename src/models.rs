
use mesh::*;
use std;

pub fn create_icosahedron() -> Result<StaticMesh, Error>
{
    let x = 0.525731112119133606;
    let z = 0.850650808352039932;

    let positions = vec!(
       -x, 0.0, z, x, 0.0, z, -x, 0.0, -z, x, 0.0, -z,
       0.0, z, x, 0.0, z, -x, 0.0, -z, x, 0.0, -z, -x,
       z, x, 0.0, -z, x, 0.0, z, -x, 0.0, -z, -x, 0.0
    );
    let indices = vec!(
       0,1,4, 0,4,9, 9,4,5, 4,8,5, 4,1,8,
       8,1,10, 8,10,3, 5,8,3, 5,3,2, 2,3,7,
       7,3,10, 7,10,6, 7,6,11, 11,6,0, 0,6,1,
       6,10,1, 9,11,0, 9,2,11, 9,5,2, 7,11,2
    );

    let mesh = StaticMesh::create(indices, att!["position" => (positions, 3)])?;
    Ok(mesh)
}

pub fn create_sphere(subdivisions: usize) -> Result<StaticMesh, Error>
{
    let mesh = create_icosahedron()?;
    // TODO: Subdivide icosahedron
    Ok(mesh)
}

pub fn create_cylinder(x_subdivisions: usize, angle_subdivisions: usize) -> Result<StaticMesh, Error>
{
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for i in 0..x_subdivisions+1 {
        let x = i as f32 / x_subdivisions as f32;
        for j in 0..angle_subdivisions {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

            positions.push(x);
            positions.push(angle.cos());
            positions.push(angle.sin());
        }
    }
    for i in 0..x_subdivisions as u32 {
        for j in 0..angle_subdivisions as u32 {
            indices.push(i * angle_subdivisions as u32 + j);
            indices.push(i * angle_subdivisions as u32 + (j+1)%angle_subdivisions as u32);
            indices.push((i+1) * angle_subdivisions as u32 + (j+1)%angle_subdivisions as u32);

            indices.push(i * angle_subdivisions as u32 + j);
            indices.push((i+1) * angle_subdivisions as u32 + (j+1)%angle_subdivisions as u32);
            indices.push((i+1) * angle_subdivisions as u32 + j);
        }
    }

    let mesh = StaticMesh::create(indices, att!["position" => (positions, 3)])?;
    Ok(mesh)
}

pub fn create_plane() -> Result<StaticMesh, Error>
{
    let positions: Vec<f32> = vec![
        -1.0, 0.0, -1.0,
        1.0, 0.0, -1.0,
        1.0, 0.0, 1.0,
        -1.0, 0.0, 1.0
    ];
    let normals: Vec<f32> = vec![
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0
    ];

    let indices: Vec<u32> = vec![
        0, 2, 1,
        0, 3, 2,
    ];

    let mesh = StaticMesh::create(indices, att!["position" => (positions, 3), "normal" => (normals, 3)])?;
    Ok(mesh)
}

pub fn create_connected_cube() -> Result<StaticMesh, Error>
{
    let positions: Vec<f32> = vec![
        1.0, -1.0, -1.0,
        1.0, -1.0, 1.0,
        -1.0, -1.0, 1.0,
        -1.0, -1.0, -1.0,
        1.0, 1.0, -1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0, 1.0,
        -1.0, 1.0, -1.0
    ];

    let indices: Vec<u32> = vec![
        0, 1, 2,
        0, 2, 3,
        4, 7, 6,
        4, 6, 5,
        0, 4, 5,
        0, 5, 1,
        1, 5, 6,
        1, 6, 2,
        2, 6, 7,
        2, 7, 3,
        4, 0, 3,
        4, 3, 7
    ];

    let mesh = StaticMesh::create(indices, att!["position" => (positions, 3)])?;
    Ok(mesh)
}

pub fn create_cube_as_dynamic_mesh() -> Result<DynamicMesh, Error>
{
    let positions: Vec<f32> = vec![
        1.0, -1.0, -1.0,
        1.0, -1.0, 1.0,
        -1.0, -1.0, 1.0,
        -1.0, -1.0, -1.0,
        1.0, 1.0, -1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0, 1.0,
        -1.0, 1.0, -1.0
    ];

    let indices: Vec<u32> = vec![
        0, 1, 2,
        0, 2, 3,
        4, 7, 6,
        4, 6, 5,
        0, 4, 5,
        0, 5, 1,
        1, 5, 6,
        1, 6, 2,
        2, 6, 7,
        2, 7, 3,
        4, 0, 3,
        4, 3, 7
    ];

    Ok(DynamicMesh::create(indices, positions, None))
}

pub fn create_cube() -> Result<StaticMesh, Error>
{
    let positions: Vec<f32> = vec![
        1.0, 1.0, -1.0,
        -1.0, 1.0, -1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0, 1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0, -1.0,

        -1.0, -1.0, -1.0,
        1.0, -1.0, -1.0,
        1.0, -1.0, 1.0,
        1.0, -1.0, 1.0,
        -1.0, -1.0, 1.0,
        -1.0, -1.0, -1.0,

        1.0, -1.0, -1.0,
        -1.0, -1.0, -1.0,
        1.0, 1.0, -1.0,
        -1.0, 1.0, -1.0,
        1.0, 1.0, -1.0,
        -1.0, -1.0, -1.0,

        -1.0, -1.0, 1.0,
        1.0, -1.0, 1.0,
        1.0, 1.0, 1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0, 1.0,
        -1.0, -1.0, 1.0,

        1.0, -1.0, -1.0,
        1.0, 1.0, -1.0,
        1.0, 1.0, 1.0,
        1.0, 1.0, 1.0,
        1.0, -1.0, 1.0,
        1.0, -1.0, -1.0,

        -1.0, 1.0, -1.0,
        -1.0, -1.0, -1.0,
        -1.0, 1.0, 1.0,
        -1.0, -1.0, 1.0,
        -1.0, 1.0, 1.0,
        -1.0, -1.0, -1.0
    ];
    let normals: Vec<f32> = vec![
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,

        0.0, -1.0, 0.0,
        0.0, -1.0, 0.0,
        0.0, -1.0, 0.0,
        0.0, -1.0, 0.0,
        0.0, -1.0, 0.0,
        0.0, -1.0, 0.0,

        0.0, 0.0, -1.0,
        0.0, 0.0, -1.0,
        0.0, 0.0, -1.0,
        0.0, 0.0, -1.0,
        0.0, 0.0, -1.0,
        0.0, 0.0, -1.0,

        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,

        1.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        1.0, 0.0, 0.0,

        -1.0, 0.0, 0.0,
        -1.0, 0.0, 0.0,
        -1.0, 0.0, 0.0,
        -1.0, 0.0, 0.0,
        -1.0, 0.0, 0.0,
        -1.0, 0.0, 0.0
    ];

    let uvs: Vec<f32> = vec![
        1.0, 0.0,
        0.0, 0.0,
        1.0, 1.0,
        0.0, 1.0,
        1.0, 1.0,
        0.0, 0.0,

        1.0, 0.0,
        0.0, 0.0,
        1.0, 1.0,
        0.0, 1.0,
        1.0, 1.0,
        0.0, 0.0,

        1.0, 0.0,
        0.0, 0.0,
        1.0, 1.0,
        0.0, 1.0,
        1.0, 1.0,
        0.0, 0.0,

        1.0, 0.0,
        0.0, 0.0,
        1.0, 1.0,
        0.0, 1.0,
        1.0, 1.0,
        0.0, 0.0,

        1.0, 0.0,
        0.0, 0.0,
        1.0, 1.0,
        0.0, 1.0,
        1.0, 1.0,
        0.0, 0.0,

        1.0, 0.0,
        0.0, 0.0,
        1.0, 1.0,
        0.0, 1.0,
        1.0, 1.0,
        0.0, 0.0
    ];

    let indices = (0..positions.len() as u32/3).collect();
    let mesh = StaticMesh::create(indices,
                                  att!["position" => (positions, 3), "normal" => (normals, 3), "uv_coordinate" => (uvs, 2)])?;
    Ok(mesh)
}