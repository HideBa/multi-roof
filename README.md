# Multi-Roof Assignment

A tool to convert LoD2.2 building models to LoD1.2.

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
