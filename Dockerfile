# Build and statically link the binary
FROM ekidd/rust-musl-builder:stable as builder
WORKDIR /home/rust/src/
ADD --chown=rust:rust . .
RUN cargo build --release --target x86_64-unknown-linux-musl


# Use a clean container to run the binary
FROM scratch as runner
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/meesign-server /usr/local/bin/meesign-server

ARG SERVER_PORT=1337
ARG BUILD_DATE
ARG REVISION
ARG BUILD_VERSION

LABEL org.opencontainers.image.created=${BUILD_DATE} \
      org.opencontainers.image.source="https://github.com/crocs-muni/meesign-server" \
      org.opencontainers.image.version=${BUILD_VERSION} \
      org.opencontainers.image.revision=${REVISION} \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.title="meesign-server" \
      org.opencontainers.image.description="Meesign server for threshold ECDSA signatures." \
      org.opencontainers.image.vendor="CRoCS, FI MUNI" \
      org.label-schema.docker.cmd="docker run --detach --publish ${SERVER_PORT}:${SERVER_PORT} meesign-server:latest"

EXPOSE ${SERVER_PORT}
ENTRYPOINT ["meesign-server"]
CMD ["--addr", "0.0.0.0"]
