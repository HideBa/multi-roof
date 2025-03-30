# Multi-Roof Assignment

A tool to convert LoD2.2 building models to LoD1.2.

> [!NOTE]
> The latest report of this project is [here](https://hideba.notion.site/Conversion-of-LoD2-2-to-LoD1-2-Building-Models-PhD-assignment-1c5f6b6336e080caa688e74790944a6d?pvs=4).

## Overview

This tool takes a 3D building model in LoD2.2 (detailed roof structures) and converts it to LoD1.2 (block-shaped representation with flat roof). The conversion process involves:

1. Identifying ground, wall, and roof surfaces
2. Calculating an appropriate roof height based on existing roof surfaces
3. Removing existing wall and roof surfaces
4. Extruding the ground footprint to the calculated height
5. Generating the output LoD1.2 model

## Building the Tool

### Prerequisites

- Rust and Cargo (install from [rustup.rs](https://rustup.rs))

### Building and running the tool

```bash
# Clone the repository
git clone git@github.com:HideBa/multi-roof-assignment.git
cd multi-roof-assignment

# Build the project
cargo build --release
./target/release/lodconv --input ./yourdata.obj --output ./yourdata_lod1.2.obj
# Or use make to run the conversion under the data/input and data/output folders (debug mode)
make conv
```

### Command Line Options

- `--input`: Path to the input OBJ file (LoD2.2 building)
- `--output`: Path to save the output OBJ file (LoD1.2 building)
- `--verbose`: Enable verbose logging

## Dependencies

- `cgmath`: For vector and matrix operations
- `clap`: For command-line argument handling
- `thiserror`: For error handling

## Module structure

```
src/
├── lib.rs               # Library entry point, exports main modules and functions
│   ├── convert_lod()    # Main conversion function
│   └── Constants        # EPSILON, thresholds for wall angles, ground height, etc.
│
├── main.rs              # CLI application using clap for argument parsing
│   ├── Args struct      # CLI argument definitions
│   └── Command enum     # Subcommands (Convert)
│
├── error.rs             # Error handling
│   ├── Error enum       # Custom error types (IO, CommandLine, Rerun)
│   └── Result type      # Type alias for Result with Error
│
├── primitives.rs        # Basic geometric primitives
│   ├── SurfaceType enum # Classification for surfaces (Ground, Wall, Roof, Unknown)
│   ├── Vertex struct    # 3D point with ID
│   └── Face struct      # Building face with methods for:
│       ├── normal()                # Calculate face normal vector
│       ├── z_range(), height()     # Height calculations
│       ├── projected_area()        # Area calculations
│       └── is_adjacent_to()        # Face adjacency checking
│
└── model.rs             # Core building model implementation
    ├── Model struct     # Building model with vertices and faces
    └── impl Model       # Implementation with methods for:
        ├── read_obj(), write_obj() # File I/O
        ├── build_adjacency()       # Build adjacency information
        ├── classify_surfaces()     # Mark surfaces as ground, wall, or roof
        ├── calculate_lod1_2_height() # Calculate height for LoD1.2
        ├── extrude_to_lod1()       # Create extruded model
        ├── to_lod1_2()             # Convert LoD2.2 to LoD1.2
        └── visualize()             # Visualization with Rerun
```

## Disclaimers

- The program isn't tested with enough dataset. Only tested with given dataset.
- The program assumes that there are no outliers especially in min_z value.
- The program doesn't care about the orientation of faces' vertices. Ideally follow right hand rule but it's not handled yet. Recommend to visualize both sides of faces when you visualize with viewer.
