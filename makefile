# Makefile for lodconv project

# Default target
all: build

# Build the project
build:
	cargo build

# Run the project with conversion
conv:
	cargo run -- convert --input ./data/input/simple.obj --output ./data/output/simple.obj
	cargo run -- convert --input ./data/input/beeb.obj --output ./data/output/beeb.obj
	cargo run -- convert --input ./data/input/bk.obj --output ./data/output/bk.obj
	cargo run -- convert --input ./data/input/otb.obj --output ./data/output/otb.obj

# Run linting and formatting checks
check:
	cargo fmt -- --check
	cargo clippy

# Format the code
fmt:
	cargo fmt

# Clean the project
clean:
	cargo clean

# Run tests
test:
	cargo test

.PHONY: all build conv check fmt clean test
