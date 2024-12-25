FROM rust
RUN apt-get update -qy && apt-get install -qy nmap
RUN rustup component add rustfmt clippy