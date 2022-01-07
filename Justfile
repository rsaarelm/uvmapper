# Unit test everything.
test:
    @cargo fmt --all -- --check
    @cargo test --all

# Generate documentation.
doc:
    @cargo doc

# Reformat code.
fmt:
    @cargo fmt --all

# Build all images (must have ULTIMA_V_PATH variable set)
make-images:
    @cargo run --release
    mkdir -p img
    mv *.png img/
    optipng img/*.png
