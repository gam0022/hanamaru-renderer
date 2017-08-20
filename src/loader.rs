use std::fs::File;
use std::io::{BufReader, BufRead};

use vector::Vector3;
use matrix::Matrix44;
use scene::{Mesh, Face};
use material::Material;

pub struct ObjLoader;

impl ObjLoader {
    pub fn load(path: &str, matrix: Matrix44, material: Material) -> Mesh {
        let mut mesh = Mesh {
            vertexes: vec![],
            faces: vec![],
            material: material,
        };

        let f = File::open(path).unwrap();
        let file = BufReader::new(&f);
        for (_, line) in file.lines().enumerate() {
            let l = line.unwrap();
            let split_line: Vec<&str> = l.split(" ").collect();
            match split_line[0] {
                "v" => {
                    let local_vertex = Vector3::new(
                        split_line[1].parse::<f64>().unwrap(),
                        split_line[2].parse::<f64>().unwrap(),
                        split_line[3].parse::<f64>().unwrap(),
                    );
                    let world_vertex = matrix * local_vertex;
                    mesh.vertexes.push(world_vertex);
                }
                "f" => {
                    let v1: Vec<&str> = split_line[1].split("/").collect();
                    let v2: Vec<&str> = split_line[2].split("/").collect();
                    let v3: Vec<&str> = split_line[3].split("/").collect();
                    mesh.faces.push(Face {
                        v0: v1[0].parse::<usize>().unwrap() - 1,
                        v1: v2[0].parse::<usize>().unwrap() - 1,
                        v2: v3[0].parse::<usize>().unwrap() - 1,
                    });
                }
                _ => {}
            }
        }

        mesh
    }
}
