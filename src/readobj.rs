use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Clone)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub struct Triangle {
    pub v1: Vec3,
    pub v2: Vec3,
    pub v3: Vec3,
    pub n1: Vec3,
    pub n2: Vec3,
    pub n3: Vec3,
}

pub fn read_obj_file<P>(filename: P) -> Result<(Vec<Vec3>, Vec<Vec3>, Vec<Triangle>), io::Error>
where
    P: AsRef<Path>,
{
    let file = File::open(&filename)?;
    let reader = io::BufReader::new(file);

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut triangles = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" => {
                if parts.len() < 4 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid vertex data: {}", line)));
                }
                let x = parts[1].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid x coordinate"))?;
                let y = parts[2].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid y coordinate"))?;
                let z = parts[3].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid z coordinate"))?;
                vertices.push(Vec3 { x, y, z });
            }
            "vn" => {
                if parts.len() < 4 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid normal data: {}", line)));
                }
                let x = parts[1].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid x coordinate"))?;
                let y = parts[2].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid y coordinate"))?;
                let z = parts[3].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid z coordinate"))?;
                normals.push(Vec3 { x, y, z });
            }
            "f" => {
                if parts.len() < 4 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid face data: {}", line)));
                }
                let mut vertex_indices = Vec::new();
                let mut normal_indices = Vec::new();
                for part in &parts[1..] {
                    let indices: Vec<&str> = part.split('/').collect();
                    if indices.len() < 1 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid face index data: {}", line)));
                    }
                    let vertex_idx = indices[0].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid vertex index"))?;
                    vertex_indices.push(vertex_idx);

                    // Handle normal indices
                    if indices.len() >= 3 {
                        let normal_idx = indices[2].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid normal index"))?;
                        normal_indices.push(normal_idx);
                    } else {
                        // Handle case where normal index is not provided
                        normal_indices.push(0); // Default normal or handle differently
                    }
                }

                let n1 = normal_indices.get(0).and_then(|&idx| if idx > 0 { normals.get(idx - 1).cloned() } else { None }).unwrap_or(Vec3 { x: 0.0, y: 0.0, z: 0.0 });
                let n2 = normal_indices.get(1).and_then(|&idx| if idx > 0 { normals.get(idx - 1).cloned() } else { None }).unwrap_or(Vec3 { x: 0.0, y: 0.0, z: 0.0 });
                let n3 = normal_indices.get(2).and_then(|&idx| if idx > 0 { normals.get(idx - 1).cloned() } else { None }).unwrap_or(Vec3 { x: 0.0, y: 0.0, z: 0.0 });

                let v1 = vertex_indices.get(0).and_then(|&idx| if idx > 0 { vertices.get(idx - 1).cloned() } else { None }).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid vertex index: {}", vertex_indices.get(0).unwrap_or(&0))))?;
                let v2 = vertex_indices.get(1).and_then(|&idx| if idx > 0 { vertices.get(idx - 1).cloned() } else { None }).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid vertex index: {}", vertex_indices.get(1).unwrap_or(&0))))?;
                let v3 = vertex_indices.get(2).and_then(|&idx| if idx > 0 { vertices.get(idx - 1).cloned() } else { None }).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid vertex index: {}", vertex_indices.get(2).unwrap_or(&0))))?;

                triangles.push(Triangle { v1, v2, v3, n1, n2, n3 });
            }
            _ => {}
        }
    }

    Ok((vertices, normals, triangles))
}

pub fn prepare_mesh_data(triangles: &[Triangle]) -> (Vec<f32>, Vec<f32>, Vec<i32>) {
    let mut vertex_data = Vec::with_capacity(triangles.len() * 9); // 3 vertices * 3 components per vertex
    let mut normal_data = Vec::with_capacity(triangles.len() * 9); // 3 normals * 3 components per normal
    let mut index_data = Vec::with_capacity(triangles.len() * 3); // 3 indices per triangle

    for (i, triangle) in triangles.iter().enumerate() {
        let base_index = (i * 3) as i32;

        // Add vertices
        vertex_data.extend_from_slice(&[
            triangle.v1.x as f32, triangle.v1.y as f32, triangle.v1.z as f32,
            triangle.v2.x as f32, triangle.v2.y as f32, triangle.v2.z as f32,
            triangle.v3.x as f32, triangle.v3.y as f32, triangle.v3.z as f32,
        ]);

        // Add normals
        normal_data.extend_from_slice(&[
            triangle.n1.x as f32, triangle.n1.y as f32, triangle.n1.z as f32,
            triangle.n2.x as f32, triangle.n2.y as f32, triangle.n2.z as f32,
            triangle.n3.x as f32, triangle.n3.y as f32, triangle.n3.z as f32,
        ]);

        // Add indices
        index_data.extend_from_slice(&[base_index, base_index + 1, base_index + 2]);
    }

    (vertex_data, normal_data, index_data)
}
