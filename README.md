# SPF - Statis Pro Football

This is a Rust workspace project that implements a server for running Statis Pro Football games.

## Structure

This workspace contains two crates:
- **spf** - The main server application (actix-web server)
- **spf_macros** - Procedural macros used by the server

## Development

### Using DevContainer (Recommended)

This project includes a DevContainer configuration for a consistent development environment:

1. Install [Docker](https://www.docker.com/products/docker-desktop) and [VS Code](https://code.visualstudio.com/)
2. Install the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
3. Open this project in VS Code
4. Click "Reopen in Container" when prompted (or use Command Palette: "Dev Containers: Reopen in Container")

The devcontainer includes:
- Latest Rust toolchain with clippy and rustfmt
- rust-analyzer for IDE support
- Zsh with Oh My Zsh
- Git and common CLI tools
- Debugger support (lldb)
- Port forwarding for 8080 and 3000

### Local Development

If not using DevContainer:

#### Building

```bash
cargo build
```

#### Running

```bash
cargo run
```

#### Testing

```bash
cargo test
```
