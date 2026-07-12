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

## API Documentation (Swagger / OpenAPI)

The OpenAPI spec is generated directly from the server code (via `utoipa`); there is no
hand-maintained `swagger.yaml`. Once the server is running (`cargo run`, listening on
`127.0.0.1:8080`), you can access:

- **Swagger UI** (interactive docs): [http://127.0.0.1:8080/swagger-ui/](http://127.0.0.1:8080/swagger-ui/)
- **Raw OpenAPI JSON spec:** [http://127.0.0.1:8080/api-docs/openapi.json](http://127.0.0.1:8080/api-docs/openapi.json)

The raw spec can be imported into other tools (Postman, code generators, etc.). For example, to
save it to a file:

```bash
curl http://127.0.0.1:8080/api-docs/openapi.json -o openapi.json
```

Endpoints are grouped into `game`, `offense`, `defense`, and `players` tags in the UI.

## Live Events (WebSocket)

In addition to the REST API, the server pushes live game events over a **read-only**
WebSocket at `GET /game/ws`. On connect the client immediately receives a snapshot of the
current game state, then a JSON frame for every subsequent change (lineup set, next play
type selected, play run, …). Commands are still issued via REST; the socket is for
notifications only. Returns `409 Conflict` if no game is in progress.

You can smoke-test it with [`websocat`](https://github.com/vi/websocat) (a command-line
WebSocket client — install with `cargo install websocat`):

```bash
# 1. Start a game first (via Swagger UI or curl), then connect:
websocat ws://127.0.0.1:8080/game/ws
```

Each message is a tagged JSON object, e.g.:

```json
{"event":"GameStarted","data":{"state":{ "quarter":1, "possession":"Away", "...":"..." }}}
```

Drive the game via REST (e.g. `POST /game/nexttype`, `POST /game/play`) in another terminal
and watch the corresponding `NextPlayTypeSet` / `PlayRun` events arrive on the socket.
