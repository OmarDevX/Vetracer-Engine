use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Clone)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

struct Triangle {
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
}

fn read_obj_file<P>(filename: P) -> Vec<Triangle>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).expect("Unable to open file");
    let reader = io::BufReader::new(file);

    let mut vertices: Vec<Vec3> = Vec::new();
    let mut triangles: Vec<Triangle> = Vec::new();

    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        if parts.len() > 0 {
            match parts[0] {
                "v" => {
                    let vertex = Vec3 {
                        x: parts[1].parse().expect("Invalid x coordinate"),
                        y: parts[2].parse().expect("Invalid y coordinate"),
                        z: parts[3].parse().expect("Invalid z coordinate"),
                    };
                    vertices.push(vertex);
                }
                "f" => {
                    let mut vertex_indices: Vec<usize> = Vec::new();
                    for part in &parts[1..] {
                        let indices: Vec<&str> = part.split('/').collect();
                        vertex_indices.push(indices[0].parse().expect("Invalid vertex index"));
                    }
                    let v1 = vertices[vertex_indices[0] - 1].clone();
                    let v2 = vertices[vertex_indices[1] - 1].clone();
                    let v3 = vertices[vertex_indices[2] - 1].clone();
                    triangles.push(Triangle { v1, v2, v3 });
                }
                _ => {}
            }
        }
    }

    triangles
}
