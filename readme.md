# Setup

Build

    cargo build
    cargo run

Firewall rules:

    sudo firewall-cmd --permanent --add-port=8091/tcp
    sudo firewall-cmd --reload

Access:

    http://localhost:8091

## Architecture

![](./arch.drawio.png)

## Questions:

- Is it a good idea to share a common state within the whole application? Like `SharedHandle`
- `Arc<Mutex<Stats>>` to protect the stats?