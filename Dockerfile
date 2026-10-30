FROM rust:1-alpine3.24 AS builder

# Install OpenSSL headers and C compilation tools required by alpine
RUN apk add --no-cache pkgconfig openssl-dev openssl-libs-static musl-dev build-base
ENV OPENSSL_STATIC=1

# install the dependencies needed for crates to work
#RUN apt-get update && apt-get install -y libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

# install cargo watch 
RUN cargo install cargo-watch

WORKDIR /hash_route

COPY . .

RUN cargo fetch

CMD ["cargo", "watch", "-w", "crates", "-x", "run -p hashroute --bin app"]

