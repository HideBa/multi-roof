pub mod error;
pub mod model;
pub mod primitives;

pub use error::{Error, Result};
pub use model::Model;
pub use primitives::{Face, SurfaceType, Vertex};

use std::path::Path;

pub const EPSILON: f64 = 1e-6; // epsilon for floating point comparison
pub const WALL_ANGLE_THRESHOLD: f64 = 0.01; // angle threshold for wall against the up vector
pub const GROUND_HEIGHT_THRESHOLD: f64 = 1.0; // height threshold for ground. Assuming all ground surfaces vertices are within 1.0 m of min z value
pub const ROOF_HEIGHT_PERCENTILE: f64 = 0.7; // percentile of roof height to use for LoD1.2 height. Default is 70% which follows 3DBAG decisions

/// Convert a LoD2.2 OBJ file to a LoD1.2 OBJ file
pub fn convert_lod(input_path: &Path, output_path: &Path) -> Result<()> {
    // Initialize rerun

    let mut model = Model::read_obj(input_path)?;

    // Debug
    // =========================
    // Visualize the model
    // let mut rec = rerun::RecordingStreamBuilder::new("lodconv.rrd").spawn()?;
    // println!("Starting visualization viewer...");
    // if visualize {
    //     model.visualize(&mut rec, "lod2_2")?;
    //     // Start the viewer
    //     println!("Visualization ready. Press Ctrl+C to exit.");
    // }
    // =========================
    // Convert the model from LoD2.2 to LoD1.2
    model.to_lod1_2()?;

    // Debug
    // =========================
    // if visualize {
    //     model.visualize(&mut rec, "lod1_2")?;
    // }
    // =========================

    // Write the output OBJ file
    model.write_obj(output_path)?;

    Ok(())
}
