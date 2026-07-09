# Development Environment & OpenAPI (utoipa)

Durable reference for the local development setup, the ports/tooling, and — the bulk of this
doc — how the OpenAPI/Swagger spec is generated from code via `utoipa`.

For the HTTP handler layout itself, see [`workspace-structure.md`](workspace-structure.md)
(`webendpoint.rs`). For build/run commands, see [`commands.md`](commands.md).

---

## Development environment

- **Recommended:** Use the provided DevContainer.
- **Ports:** The server runs on **8080**; a companion front-end (if present) runs on **3000**.
  Both are forwarded by the devcontainer.
- The devcontainer configures rust-analyzer to run Clippy on every save
  (`"rust-analyzer.check.command": "clippy"`), so Clippy is the live feedback loop during
  development.

---

## API exploration

- Browse the live API at `http://127.0.0.1:8080/swagger-ui/` (raw spec at
  `/api-docs/openapi.json`).
- A Postman collection (`spf.postman_collection.json`) at the workspace root also documents all
  HTTP endpoints.

---

## OpenAPI / Swagger (utoipa)

The API spec is *generated from code*, not hand-written. There is no `swagger.yaml`; it was
retired in favour of `utoipa` + `utoipa-actix-web` + `utoipa-swagger-ui` (see `spf/Cargo.toml`).
Key conventions:

- **Handlers** in `webendpoint.rs` use actix attribute macros (`#[get("...")]` / `#[post("...")]`)
  plus a `#[utoipa::path(...)]` annotation, and are registered with `.service()`. The
  `actix_extras` feature lets utoipa infer the URL and path params from the actix macro, so the
  path string is written once. Routes are grouped into resource scopes (`/game`, `/offense`,
  `/defense`) via `utoipa_actix_web::scope::scope(...)`, which auto-prefix the generated paths.
- **Schemas** — types that appear in request bodies or responses derive `utoipa::ToSchema`
  (alongside serde). `ToSchema` only *describes* a type's shape for the spec; it does not replace
  serde, which still does the runtime (de)serialization. utoipa honors serde attributes
  (`rename`, `default`, `untagged`, …) when emitting schemas. Query-param structs derive
  `IntoParams`. Fidelity is tiered: deep read-only graphs (player stat structs, full play
  internals) are left opaque via `#[schema(value_type = Object)]` rather than fully annotated.
- **Manual schemas** — `OffenseCall` and `DefenseCall` (in `engine.rs`) use a custom
  `impl_deserialize!` macro instead of a real `#[serde(untagged)]`, so they have hand-written
  `ToSchema` impls that emit a `oneOf` of the inner call structs. Adding a new variant means
  updating that `oneOf`.
- **`ApiDoc`** (in `webendpoint.rs`) carries base info (title/version/servers) and lists the two
  manually-implemented call enums plus their inner variant schemas under
  `components(schemas(...))`. Everything else auto-collects from `.service()` calls.
- **Wiring gotcha** — in `runserver()`, `.openapi(ApiDoc::openapi())` *replaces* the wrapped spec,
  so it must be called **before** the `.service()` calls (it prepends base info; services then add
  paths/schemas on top). Middleware (CORS) is applied via `.map(|a| a.wrap(cors))`. Swagger UI is
  mounted at `/swagger-ui/` and the raw spec at `/api-docs/openapi.json` after `split_for_parts()`.
