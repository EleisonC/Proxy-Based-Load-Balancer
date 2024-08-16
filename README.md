# Proxy-Based-Load-Balancer
Proxy Based Load Balancer with Adaptive Decision Engine


## Setup & Building

Before proceeding, ensure that the necessary services are running.

### Prerequisites

- Ensure the the example servers are running in the example_servers file

### Build Steps

```bash
cargo install cargo-watch
cd proxy_load_balancer
cargo build
cd ..
```

## Run server locally (Manually)

```bash
# Navigate to the project directory
cd proxy_load_balancer

# Run the load balancer
cargo run
```

##  Using Postman

To interact with the load balancer via Postman:

- Set the address to http://127.0.0.1:4000.

- Make your API calls to the auth service through this endpoint.
