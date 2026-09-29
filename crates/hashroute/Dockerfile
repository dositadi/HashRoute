FROM rust:1.98-slim-trixie as builder

WORKDIR /hashroute

COPY . .

RUN Cargo install --path .

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*

WORKDIR /bin

COPY --from=builder /hashroute /bin

CMD [ "bin" ]



