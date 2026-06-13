# Uses the official Rust image which includes cargo and rustc
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

COPY . .

# Build the release binary
RUN cargo build --release

# Run the compiled binary
CMD ["./target/release/confess"]
