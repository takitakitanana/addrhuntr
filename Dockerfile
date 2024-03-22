# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/addrhuntr

# Copy the Rust project files to the container
COPY . .

# Build the Rust project
RUN cargo build --release

# Move the binary from the target/release directory to the current working directory
RUN mv target/release/addrhuntr .

# When the container is run, it will execute the binary "addrhuntr"
ENTRYPOINT ["./addrhuntr"]
CMD ["--help"]
