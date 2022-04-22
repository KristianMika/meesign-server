# Meesign Server

## Regular Build

```bash
docker build [--release]
```

## Build for Docker Image

### Static Build for Docker Image

Rust compiler links Rust libraries statically but uses dynamic linking for C libraries. To run the binary in a Docker container, it is necessary (almost, this is the easier way) to link the binary statically. We are using [rust-musl-builder](https://github.com/emk/rust-musl-builder) for this job.

```bash
alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder'
rust-musl-builder cargo build --release
```


### Docker Image Build

```bash
docker build . --tag meesign-server:latest
```

### Run in a Container Locally

```bash
docker run --detach --publish 1337:1337 meesign-server:latest
```