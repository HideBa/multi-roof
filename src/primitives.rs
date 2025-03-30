use crate::EPSILON;
use cgmath::{InnerSpace, Point3, Vector3};

/// Surface type classification
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceType {
    Ground,
    Wall,
    Roof,
    Unknown, // default value
}

/// A vertex in the model
#[derive(Debug, Clone)]
pub struct Vertex {
    pub point: Point3<f64>,
    pub id: usize,
}

/// A face in the model
#[derive(Debug, Clone)]
pub struct Face {
    pub vertex_ids: Vec<usize>, // IDs referencing vertices in the model
    pub surface_type: SurfaceType,
    pub adjacent_faces: Vec<usize>, // Indices of adjacent faces
}

impl Face {
    /// Create a new face from vertex IDs
    pub fn new(vertex_ids: Vec<usize>) -> Self {
        Face {
            vertex_ids,
            surface_type: SurfaceType::Unknown,
            adjacent_faces: Vec::new(),
        }
    }

    /// Calculate the normal vector of the face
    pub fn normal(&self, vertices: &[Vertex]) -> Vector3<f64> {
        if self.vertex_ids.len() < 3 {
            return Vector3::new(0.0, 0.0, 1.0); // Default normal for degenerate faces
        }

        // Use the first three vertices to calculate a normal
        let p0 = &vertices[self.vertex_ids[0]].point;
        let p1 = &vertices[self.vertex_ids[1]].point;
        let p2 = &vertices[self.vertex_ids[2]].point;

        // Calculate vectors along two edges
        let v1 = Vector3::new(p1.x - p0.x, p1.y - p0.y, p1.z - p0.z);
        let v2 = Vector3::new(p2.x - p0.x, p2.y - p0.y, p2.z - p0.z);

        // Cross product gives normal vector
        let normal = v1.cross(v2);

        // Normalize the vector, return default if degenerate
        if normal.magnitude() < EPSILON {
            Vector3::new(0.0, 0.0, 1.0)
        } else {
            normal.normalize()
        }
    }

    /// Calculate the minimum and maximum Z values of the face
    pub fn z_range(&self, vertices: &[Vertex]) -> (f64, f64) {
        if self.vertex_ids.is_empty() {
            return (0.0, 0.0);
        }

        let mut min_z = vertices[self.vertex_ids[0]].point.z;
        let mut max_z = min_z;

        for &id in &self.vertex_ids {
            let z = vertices[id].point.z;
            if z < min_z {
                min_z = z;
            }
            if z > max_z {
                max_z = z;
            }
        }

        (min_z, max_z)
    }

    /// Calculate the height of the face (max_z - min_z)
    pub fn height(&self, vertices: &[Vertex]) -> f64 {
        let (min_z, max_z) = self.z_range(vertices);
        max_z - min_z
    }

    /// Calculate the area of the face projected onto the XY plane
    pub fn projected_area(&self, vertices: &[Vertex]) -> f64 {
        if self.vertex_ids.len() < 3 {
            return 0.0;
        }

        // For a triangle, calculate area using cross product
        if self.vertex_ids.len() == 3 {
            let p0 = &vertices[self.vertex_ids[0]].point;
            let p1 = &vertices[self.vertex_ids[1]].point;
            let p2 = &vertices[self.vertex_ids[2]].point;

            // Create vectors in XY plane (z=0) to get projected area
            let v1 = Vector3::new(p1.x - p0.x, p1.y - p0.y, 0.0);
            let v2 = Vector3::new(p2.x - p0.x, p2.y - p0.y, 0.0);

            return v1.cross(v2).magnitude() * 0.5;
        }

        // For polygons with more than 3 vertices, decompose into triangles
        // using the first vertex as a base
        let p0 = &vertices[self.vertex_ids[0]].point;
        let mut total_area = 0.0;

        for i in 1..(self.vertex_ids.len() - 1) {
            let p1 = &vertices[self.vertex_ids[i]].point;
            let p2 = &vertices[self.vertex_ids[i + 1]].point;

            // Create vectors in XY plane (z=0) to get projected area
            let v1 = Vector3::new(p1.x - p0.x, p1.y - p0.y, 0.0);
            let v2 = Vector3::new(p2.x - p0.x, p2.y - p0.y, 0.0);

            total_area += v1.cross(v2).magnitude() * 0.5;
        }

        total_area
    }

    /// Check if this face is adjacent to another face
    pub fn is_adjacent_to(&self, other: &Face) -> bool {
        // Two faces are adjacent if they share at least two vertex IDs
        let mut shared_vertices = 0;

        for &id1 in &self.vertex_ids {
            if other.vertex_ids.contains(&id1) {
                shared_vertices += 1;
                if shared_vertices >= 2 {
                    return true;
                }
            }
        }

        false
    }
}
