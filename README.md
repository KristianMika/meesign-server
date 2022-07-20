# MeeSign Server

Server-side implementation for MeeSign system.

## Usage

### Local Build

1. [Install Rust](https://www.rust-lang.org/tools/install)

2. Clone the repository:

   ```bash
   git clone https://github.com/crocs-muni/meesign-server
   ```

3. Generate server private key and certificate:

    ```bash
    bash generate_certificates.sh
    ```

4. [Prepare MeeSignHelper](https://github.com/dufkan/meesign-helper)

5. Build and run the server:

   ```bash
   cargo run
   ```

### Run in a Docker Container

1. Generate server private key and certificate:

    ```bash
    bash generate_certificates.sh
    ```

2. Run from in a container
   ```bash
   docker run --detach --publish 1337:1337 --volume `pwd`/server-key.pem:/meesign/server-key.pem --volume `pwd`/server-cert.pem:/meesign/server-cert.pem  meesign-server:latest 
   ```
