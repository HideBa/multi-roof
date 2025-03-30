use crate::error::{Error, Result};
use crate::primitives::{Face, SurfaceType, Vertex};
use crate::{EPSILON, GROUND_HEIGHT_THRESHOLD, WALL_ANGLE_THRESHOLD};
use cgmath::{InnerSpace, Point3, Vector3};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::str::FromStr;
use std::{collections::HashMap, path::Path};

/// A 3D building model
#[derive(Debug, Clone)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
}

impl Model {
    /// Create a new model with the given vertices and faces
    pub fn new(vertices: Vec<Vertex>, faces: Vec<Face>) -> Self {
        let mut model = Model { vertices, faces };
        model.build_adjacency();
        model
    }

    /// Load a model from an OBJ file
    pub fn read_obj(path: &Path) -> Result<Self> {
        // Resolve path: if relative, use it relative to current directory
        let resolved_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            // Normalize the path to remove any "./" prefixes
            std::env::current_dir()
                .map_err(Error::Io)?
                .join(path)
                .canonicalize()
                .map_err(|e| {
                    Error::Io(std::io::Error::new(
                        e.kind(),
                        format!("Failed to resolve path {}: {}", path.display(), e),
                    ))
                })?
        };

        let file = File::open(&resolved_path).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to open file {}: {}", resolved_path.display(), e),
            ))
        })?;

        let reader = BufReader::new(file);

        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(Error::Io)?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "v" => {
                    if parts.len() < 4 {
                        return Err(Error::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Not enough components for vertex at line {}",
                                line_number + 1
                            ),
                        )));
                    }

                    let x = f64::from_str(parts[1]).map_err(|_| {
                        Error::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Invalid x coordinate: {} at line {}",
                                parts[1],
                                line_number + 1
                            ),
                        ))
                    })?;

                    let y = f64::from_str(parts[2]).map_err(|_| {
                        Error::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Invalid y coordinate: {} at line {}",
                                parts[2],
                                line_number + 1
                            ),
                        ))
                    })?;

                    let z = f64::from_str(parts[3]).map_err(|_| {
                        Error::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Invalid z coordinate: {} at line {}",
                                parts[3],
                                line_number + 1
                            ),
                        ))
                    })?;

                    vertices.push(Vertex {
                        point: Point3::new(x, y, z),
                        id: vertices.len(),
                    });
                }
                "f" => {
                    if parts.len() < 4 {
                        return Err(Error::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Face must have at least 3 vertices at line {}",
                                line_number + 1
                            ),
                        )));
                    }

                    let mut vertex_ids = Vec::new();
                    for part in &parts[1..] {
                        // Extract just the vertex index (ignore texture/normal indices)
                        let vertex_str = part.split('/').next().unwrap_or("");
                        let index = usize::from_str(vertex_str).map_err(|_| {
                            Error::Io(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!(
                                    "Invalid vertex index: {} at line {}",
                                    vertex_str,
                                    line_number + 1
                                ),
                            ))
                        })?;

                        // OBJ indices are 1-based, convert to 0-based
                        if index == 0 || index > vertices.len() {
                            return Err(Error::Io(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Invalid vertex index at line {}", line_number + 1),
                            )));
                        }

                        vertex_ids.push(index - 1);
                    }

                    faces.push(Face::new(vertex_ids));
                }
                _ => {
                    // Ignore other types like normals, textures, etc.
                    continue;
                }
            }
        }

        println!(
            "Debug: Finished parsing OBJ file. Found {} vertices and {} faces",
            vertices.len(),
            faces.len()
        );
        Ok(Self::new(vertices, faces))
    }

    /// Write the model to an OBJ file
    pub fn write_obj(&self, path: &Path) -> Result<()> {
        let resolved_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            // Normalize the path to remove any "./" prefixes
            std::env::current_dir()
                .map_err(Error::Io)?
                .join(path)
                .canonicalize()
                .map_err(|e| {
                    Error::Io(std::io::Error::new(
                        e.kind(),
                        format!("Failed to resolve path {}: {}", path.display(), e),
                    ))
                })?
        };

        println!("Writing model to {}", resolved_path.display());

        let mut file = File::create(&resolved_path).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create file {}: {}", resolved_path.display(), e),
            ))
        })?;

        // Write header
        writeln!(file, "# Converted LoD1.2 model").map_err(Error::Io)?;

        // Create collections for OBJ vertices and faces
        let mut obj_vertices: Vec<Point3<f64>> = Vec::new();
        let mut obj_faces: Vec<Vec<usize>> = Vec::new();

        // Process faces and collect unique vertices
        for face in &self.faces {
            let mut face_indices = Vec::new();

            for &vertex_id in &face.vertex_ids {
                // For each vertex, find if it already exists in our output vertices
                let vertex = &self.vertices.iter().find(|v| v.id == vertex_id).unwrap();
                // let vertex = &self.vertices[vertex_id];
                let point = &vertex.point;

                let mut vertex_index = None;

                for (i, existing_vertex) in obj_vertices.iter().enumerate() {
                    // Compare using epsilon for floating point precision
                    if (existing_vertex.x - point.x).abs() < EPSILON
                        && (existing_vertex.y - point.y).abs() < EPSILON
                        && (existing_vertex.z - point.z).abs() < EPSILON
                    {
                        vertex_index = Some(i);
                        break;
                    }
                }

                let index = match vertex_index {
                    Some(idx) => idx,
                    None => {
                        let idx = obj_vertices.len();
                        obj_vertices.push(*point);
                        idx
                    }
                };

                face_indices.push(index);
            }

            obj_faces.push(face_indices);
        }

        // Write vertices
        for vertex in &obj_vertices {
            writeln!(file, "v {} {} {}", vertex.x, vertex.y, vertex.z).map_err(Error::Io)?;
        }

        // Write faces
        for face in &obj_faces {
            write!(file, "f").map_err(Error::Io)?;
            for &index in face {
                // OBJ indices are 1-based
                write!(file, " {}", index + 1).map_err(Error::Io)?;
            }
            writeln!(file).map_err(Error::Io)?;
        }

        Ok(())
    }

    /// Build the adjacency information for faces
    fn build_adjacency(&mut self) {
        for i in 0..self.faces.len() {
            for j in 0..self.faces.len() {
                if i != j && self.faces[i].is_adjacent_to(&self.faces[j]) {
                    self.faces[i].adjacent_faces.push(j);
                }
            }
        }
    }

    /// Find all faces with the lowest Z value (ground candidates)
    fn mark_ground_faces(&mut self) -> Vec<usize> {
        if self.faces.is_empty() {
            return Vec::new();
        }

        // Step 1: Find the minimum Z value across all faces
        let mut min_z = (0, f64::MAX); // (face_id, min_z)
        for (i, face) in self.faces.iter().enumerate() {
            let (face_min_z, _) = face.z_range(&self.vertices);
            if face_min_z < min_z.1 {
                min_z = (i, face_min_z);
            }
        }

        // Step 2: Mark surfaces whose normal is almost horizontal and it's z value is within 1.0 m of the minimum z value
        let up = Vector3::new(0.0, 0.0, 1.0);

        let mut ground_faces = HashSet::new();
        for (i, face) in self.faces.iter().enumerate() {
            let normal = face.normal(&self.vertices);
            // dot product with up vector (z-axis)
            let dot_with_up = normal.dot(up).abs();
            // difference with minimum z value, assuming all ground surfaces vertices are within 1.0 m of min z value
            let diff_with_minz = (face.z_range(&self.vertices).0 - min_z.1).abs();

            // if the dot product is close to 1.0, the face is almost horizontal
            // and the difference with the minimum z value is within 1.0 m
            if dot_with_up > (1.0 - WALL_ANGLE_THRESHOLD)
                && diff_with_minz < GROUND_HEIGHT_THRESHOLD
            {
                ground_faces.insert(i);
            }
        }
        println!("Minimum z value: {}", min_z.1);

        // Debug: print the ground faces
        // =====================================
        println!("Ground faces: {:?}", ground_faces);
        // =====================================

        self.faces
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| ground_faces.contains(i))
            .for_each(|(_, face)| face.surface_type = SurfaceType::Ground);

        ground_faces.into_iter().collect()
    }

    /// Classify all faces as ground, wall, or roof based on orientation
    fn classify_surfaces(&mut self) {
        // First identify ground faces
        self.mark_ground_faces();

        // Debug: count the number of ground faces
        // =====================================
        let ground_face_count = self
            .faces
            .iter()
            .filter(|face| face.surface_type == SurfaceType::Ground)
            .count();
        println!("Ground faces: {}", ground_face_count);
        // =====================================

        // Then identify walls (normals approximately horizontal)
        let up = Vector3::new(0.0, 0.0, 1.0);
        for i in 0..self.faces.len() {
            if self.faces[i].surface_type != SurfaceType::Unknown {
                continue; // Skip already classified faces (ground)
            }

            let normal = self.faces[i].normal(&self.vertices);
            let dot_with_up = normal.dot(up).abs();

            if dot_with_up < WALL_ANGLE_THRESHOLD {
                // Angle close to 90 degrees with up vector
                self.faces[i].surface_type = SurfaceType::Wall;
            } else {
                // Angle close to 0 or 180 degrees with up vector
                self.faces[i].surface_type = SurfaceType::Roof;
            }
        }

        // Debug: print the faces with their surface type
        // =====================================
        println!(
            "Ground faces: {}",
            self.faces
                .iter()
                .filter(|face| face.surface_type == SurfaceType::Ground)
                .count()
        );
        println!(
            "Wall faces: {}",
            self.faces
                .iter()
                .filter(|face| face.surface_type == SurfaceType::Wall)
                .count()
        );
        println!(
            "Roof faces: {}",
            self.faces
                .iter()
                .filter(|face| face.surface_type == SurfaceType::Roof)
                .count()
        );
        // =====================================
    }

    /// Calculate the appropriate height for the LoD1.2 model based on
    /// the weighted average of roof surfaces
    fn calculate_lod1_2_height(&self) -> f64 {
        let mut total_area = 0.0;
        let mut weighted_height_sum = 0.0;

        for face in &self.faces {
            if face.surface_type == SurfaceType::Roof {
                let area = face.projected_area(&self.vertices);
                let (min_z, max_z) = face.z_range(&self.vertices);
                let middle_z = (max_z + min_z) / 2.0;

                total_area += area;
                weighted_height_sum += area * middle_z;
            }
        }

        println!("Total area: {}", total_area);
        println!("Weighted height sum: {}", weighted_height_sum);

        if total_area > EPSILON {
            weighted_height_sum / total_area
        } else {
            // If no roof surfaces found, use the maximum height of any surface
            let mut max_height = 0.0;
            for face in &self.faces {
                let (_, max_z) = face.z_range(&self.vertices);
                if max_z > max_height {
                    max_height = max_z;
                }
            }
            max_height
        }
    }

    /// Remove all faces labeled as wall or roof, and their unused vertices
    fn remove_non_ground_surfaces(&mut self) {
        // Keep only ground surfaces
        self.faces
            .retain(|face| face.surface_type == SurfaceType::Ground);

        // Collect the set of vertex IDs that are still in use
        let mut used_vertices = std::collections::HashSet::new();
        for face in &self.faces {
            for &vertex_id in &face.vertex_ids {
                used_vertices.insert(vertex_id);
            }
        }

        // Create a mapping from old vertex IDs to new vertex IDs
        let mut id_mapping = std::collections::HashMap::new();
        let mut new_vertices = Vec::new();

        for &old_id in used_vertices.iter() {
            let vertex = &self.vertices.iter().find(|v| v.id == old_id).unwrap();
            let new_id = new_vertices.len();

            // Create a new vertex with updated ID
            new_vertices.push(Vertex {
                point: vertex.point,
                id: new_id,
            });

            // Store the mapping from old ID to new ID
            id_mapping.insert(old_id, new_id);
        }

        // Update face vertex references to use the new IDs
        for face in &mut self.faces {
            face.vertex_ids = face
                .vertex_ids
                .iter()
                .map(|&old_id| *id_mapping.get(&old_id).unwrap())
                .collect();
        }

        // Replace the vertices with the pruned list
        self.vertices = new_vertices;
    }

    /// Identify and return the boundary edges of ground surfaces
    fn find_boundary_edges(&self) -> Vec<(usize, usize)> {
        // Create a map to track how many times each edge appears
        let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

        // Go through all ground faces and count edge occurrences
        for face in &self.faces {
            if face.surface_type != SurfaceType::Ground {
                continue;
            }

            let vertex_count = face.vertex_ids.len();
            for i in 0..vertex_count {
                let v1 = face.vertex_ids[i];
                let v2 = face.vertex_ids[(i + 1) % vertex_count];

                // Sort the vertices to ensure the same edge is counted correctly regardless of direction
                let edge = if v1 < v2 { (v1, v2) } else { (v2, v1) };

                *edge_count.entry(edge).or_insert(0) += 1;
            }
        }

        // Boundary edges appear exactly once
        edge_count
            .iter()
            .filter_map(|(&edge, &count)| if count == 1 { Some(edge) } else { None })
            .collect()
    }

    /// Order boundary edges to form a continuous loop
    fn order_boundary_edges(&self, edges: &[(usize, usize)]) -> Vec<usize> {
        if edges.is_empty() {
            return Vec::new();
        }

        let mut ordered_vertices = Vec::new();
        let mut remaining_edges: Vec<(usize, usize)> = edges.to_vec();

        // Start with the first edge
        let first_edge = remaining_edges.remove(0);
        ordered_vertices.push(first_edge.0);
        ordered_vertices.push(first_edge.1);

        // Continue connecting edges until we've used them all
        while !remaining_edges.is_empty() {
            let last_vertex = *ordered_vertices.last().unwrap();
            let mut found = false;

            for i in 0..remaining_edges.len() {
                let (v1, v2) = remaining_edges[i];

                if v1 == last_vertex {
                    ordered_vertices.push(v2);
                    remaining_edges.remove(i);
                    found = true;
                    break;
                } else if v2 == last_vertex {
                    ordered_vertices.push(v1);
                    remaining_edges.remove(i);
                    found = true;
                    break;
                }
            }

            if !found {
                // If we couldn't find a connecting edge, the boundary might be disconnected
                // Just add the next edge and continue
                if !remaining_edges.is_empty() {
                    println!("Remaining edges: {:?}", remaining_edges);
                    let edge = remaining_edges.remove(0);
                    ordered_vertices.push(edge.0);
                    ordered_vertices.push(edge.1);
                }
            }
        }

        // Remove duplicates while preserving order
        let mut unique_vertices = Vec::new();
        for &vertex in &ordered_vertices {
            if !unique_vertices.contains(&vertex) {
                unique_vertices.push(vertex);
            }
        }

        unique_vertices
    }

    /// Extrude the ground surface to create the LoD1.2 model
    fn extrude_to_lod1(&mut self, target_height: f64) {
        // Find boundary edges of ground surface
        let boundary_edges = self.find_boundary_edges();
        let boundary_vertices = self.order_boundary_edges(&boundary_edges);

        if boundary_vertices.is_empty() {
            return;
        }

        // Create top vertices at the target height
        let mut top_vertex_ids = Vec::new();
        for &index in &boundary_vertices {
            let original_vertex = &self.vertices[index];
            let top_point = Point3::new(
                original_vertex.point.x,
                original_vertex.point.y,
                target_height,
            );

            let new_id = self.vertices.len();
            self.vertices.push(Vertex {
                point: top_point,
                id: new_id,
            });

            top_vertex_ids.push(new_id);
        }

        // Create wall faces
        for i in 0..boundary_vertices.len() {
            let next_i = (i + 1) % boundary_vertices.len();

            let bottom_left = boundary_vertices[i];
            let bottom_right = boundary_vertices[next_i];
            let top_left = top_vertex_ids[i];
            let top_right = top_vertex_ids[next_i];

            // Create a wall face (rectangle) from the two ground vertices and two top vertices
            let wall_face = Face {
                vertex_ids: vec![bottom_left, bottom_right, top_right, top_left],
                surface_type: SurfaceType::Wall,
                adjacent_faces: Vec::new(),
            };

            self.faces.push(wall_face);
        }

        // Create roof face
        let roof_face = Face {
            vertex_ids: top_vertex_ids,
            surface_type: SurfaceType::Roof,
            adjacent_faces: Vec::new(),
        };

        self.faces.push(roof_face);

        // Update adjacency information
        self.build_adjacency();
    }

    /// Convert the model from LoD2.2 to LoD1.2
    pub fn to_lod1_2(&mut self) -> Result<()> {
        // Debug: print the number of faces and vertices
        // =====================================
        println!("Number of faces: {}", self.faces.len());
        println!("Number of vertices: {}", self.vertices.len());
        // =====================================

        // Step 1: Classify all surfaces
        self.classify_surfaces();

        // Check if we found any ground surfaces
        let ground_faces = self
            .faces
            .iter()
            .filter(|face| face.surface_type == SurfaceType::Ground)
            .collect::<Vec<_>>();
        if ground_faces.is_empty() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No ground surfaces found",
            )));
        }

        // Step 2: Calculate target height for the LoD1.2 model
        let target_height = self.calculate_lod1_2_height();
        if target_height <= 0.0 {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to calculate target height",
            )));
        }

        // Step 3: Remove all non-ground surfaces
        self.remove_non_ground_surfaces();

        // Debug: visualize the model with only ground surfaces
        // =====================================
        let mut rec = rerun::RecordingStreamBuilder::new("lodconv.rrd").spawn()?;
        self.visualize(&mut rec, "only ground")?;
        // =====================================

        // Step 4: Extrude the ground surface to the target height
        self.extrude_to_lod1(target_height);

        // Debug: print the number of faces and vertices
        // =====================================
        println!("Number of faces: {}", self.faces.len());
        println!("Number of vertices: {}", self.vertices.len());
        // =====================================
        Ok(())
    }
    pub fn visualize(&self, recording: &mut rerun::RecordingStream, name: &str) -> Result<()> {
        // Convert vertices to rerun format
        let vertex_positions: Vec<[f32; 3]> = self
            .vertices
            .iter()
            .map(|v| [v.point.x as f32, v.point.y as f32, v.point.z as f32])
            .collect();

        // Process each face into triangles (simple triangulation)
        let mut triangles = Vec::new();
        let mut triangle_colors = Vec::new();

        for face in &self.faces {
            // For faces with more than 3 vertices, triangulate as a fan
            if face.vertex_ids.len() >= 3 {
                for i in 1..(face.vertex_ids.len() - 1) {
                    // Create triangles from the first vertex and subsequent pairs
                    triangles.push([
                        face.vertex_ids[0] as u32,
                        face.vertex_ids[i] as u32,
                        face.vertex_ids[i + 1] as u32,
                    ]);

                    // Add color based on surface type
                    let color = match face.surface_type {
                        SurfaceType::Ground => [150, 75, 0, 255],     // Brown
                        SurfaceType::Wall => [200, 200, 200, 255],    // Light gray
                        SurfaceType::Roof => [220, 20, 20, 255],      // Red
                        SurfaceType::Unknown => [100, 100, 100, 255], // Dark gray
                    };
                    triangle_colors.push(color);
                }
            }
        }

        // Log the mesh with colors
        recording.log(
            format!("mesh_{}", name),
            &rerun::Mesh3D::new(vertex_positions.clone())
                .with_triangle_indices(triangles)
                .with_albedo_factor(rerun::Rgba32::from_rgb(128, 128, 128)),
        )?;

        // Create a vector of radius values for each point
        let point_radii = vec![0.1f32; vertex_positions.len()];

        // Log points separately
        recording.log(
            format!("vertices_{}", name),
            &rerun::Points3D::new(vertex_positions).with_radii(point_radii),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::Point3;

    #[test]
    fn test_calculate_normal() {
        let vertices = vec![
            Vertex {
                point: Point3::new(0.0, 0.0, 0.0),
                id: 0,
            },
            Vertex {
                point: Point3::new(1.0, 0.0, 0.0),
                id: 1,
            },
            Vertex {
                point: Point3::new(0.0, 1.0, 0.0),
                id: 2,
            },
        ];

        let face = Face::new(vec![0, 1, 2]);
        let normal = face.normal(&vertices);

        // The normal of a face in the XY plane should be (0, 0, 1)
        assert!((normal.x - 0.0).abs() < EPSILON);
        assert!((normal.y - 0.0).abs() < EPSILON);
        assert!((normal.z - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_calculate_area() {
        // Create a 1x1 triangle (half of a 1x1 square)
        let vertices = vec![
            Vertex {
                point: Point3::new(0.0, 0.0, 0.0),
                id: 0,
            },
            Vertex {
                point: Point3::new(1.0, 0.0, 0.0),
                id: 1,
            },
            Vertex {
                point: Point3::new(0.0, 1.0, 0.0),
                id: 2,
            },
        ];

        let face = Face::new(vec![0, 1, 2]);
        let area = face.projected_area(&vertices);

        // Area should be 0.5 (triangle with base 1 and height 1)
        assert!((area - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_face_adjacency() {
        // Create two triangles that share two vertices (0 and 2)
        let face1 = Face::new(vec![0, 1, 2]);
        let face2 = Face::new(vec![0, 2, 3]);

        // These faces should be adjacent
        assert!(face1.is_adjacent_to(&face2));

        // Create a triangle that doesn't share any vertices with face1
        let face3 = Face::new(vec![4, 5, 6]);

        // These faces should not be adjacent
        assert!(!face1.is_adjacent_to(&face3));
    }
}
