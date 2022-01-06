name := "uvmapper"

# Unit test everything.
test:
    @cargo fmt --all -- --check
    @cargo test --all

# Generate documentation.
doc:
    @cargo doc

# Run benchmarks (might require nightly rustc)
bench:
    @cargo bench --all

# Reformat code.
fmt:
    @cargo fmt --all

# Build release binary.
release:
    @cargo build --profile release-lto
    @strip target/release-lto/{{name}}
    @ls -lh target/release-lto/{{name}}

# Record profiling data from a debug build.
profile-debug:
    @perf record cargo run

# Record profiling data from a release build.
profile:
    @perf record cargo run --release

# Review recorded perf.data using hotspot.
review-profiling:
    @hotspot
