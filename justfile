# LITHOS Build Commands
# Usage: just <command>

# Default: build release
default: build-release

# ===== Build =====
build-release:
	cargo build --release --workspace

build-dev:
	cargo build --workspace

build-server:
	cargo build --release --bin lithos-server

build-app:
	cargo build --release --bin lithos

# ===== Check =====
check:
	cargo check --workspace --all-targets

check-dev:
	cargo check --workspace

# ===== Test =====
test:
	cargo test --workspace --all-targets

test-unit:
	cargo test --workspace --lib

test-integration:
	cargo test --workspace --test integration

test-doc:
	cargo test --workspace --doc

# ===== Benchmarks =====
bench:
	cargo bench --workspace

bench-engine:
	cargo bench --package lithos-engine-math --package lithos-engine-jobs --package lithos-engine-memory

# ===== Lint/Format =====
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

clippy-dev:
	cargo clippy --workspace --all-targets

# ===== Clean =====
clean:
	cargo clean

clean-deps:
	cargo clean -p lithos-engine-memory -p lithos-engine-jobs -p lithos-engine-math

# ===== Run =====
run:
	cargo run --release --bin lithos

run-dev:
	cargo run --bin lithos

run-server:
	cargo run --release --bin lithos-server

# ===== Generate =====
generate-docs:
	cargo doc --workspace --no-deps --document-private-items --open

generate-bindings:
	# Generate Vulkan bindings if needed
	@echo "Vulkan bindings via ash crate - no generation needed"

# ===== CI =====
ci: fmt-check clippy test

# ===== Profiling =====
profile-app:
	cargo build --release --bin lithos --features profiling
	perf record -g --call-graph=dwarf target/release/lithos

profile-server:
	cargo build --release --bin lithos-server --features profiling
	perf record -g --call-graph=dwarf target/release/lithos-server

# ===== Development Helpers =====
# Watch for changes and rebuild
watch:
	cargo watch -x "check --workspace" -x "test --workspace --lib"

# Update dependencies
update:
	cargo update --workspace

# Audit dependencies
audit:
	cargo audit

# Tree
tree:
	cargo tree --workspace -d

# Outdated
outdated:
	cargo outdated --workspace

# ===== Shader Compilation =====
shaders:
	@echo "Compiling shaders..."
	# glslangValidator -V shaders/*.vert -o shaders/spv/*.vert.spv
	# glslangValidator -V shaders/*.frag -o shaders/spv/*.frag.spv
	# glslangValidator -V shaders/*.comp -o shaders/spv/*.comp.spv
	@echo "Shader compilation not yet implemented"

# ===== Database =====
db-migrate:
	@echo "Database migrations not yet implemented"

# ===== Docker =====
docker-build:
	docker build -t lithos:latest .

docker-run:
	docker run --rm -it --gpus all lithos:latest

.PHONY: build-release build-dev build-server build-app check check-dev test test-unit test-integration test-doc bench bench-engine fmt fmt-check clippy clippy-dev clean clean-deps run run-dev run-server generate-docs generate-bindings ci profile-app profile-server watch update audit tree outdated shaders db-migrate docker-build docker-run