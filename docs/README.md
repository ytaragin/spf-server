# SPF Documentation Index

Entry point for the project's documentation. Docs are split by lifespan:

- **`design/`** — durable design & architecture references. Describe *how things are built*
  and *why*. Long-lived; grow slowly and are revised as the system evolves.
- **`plans/`** — work plans and staged roadmaps. Describe *what we will do next*.
  Time-ordered and consumed as work lands; may be archived once complete.

> Rule of thumb: if it answers "how does this work / why is it this way," it belongs in
> `design/`. If it answers "what's the next step / in what order," it belongs in `plans/`.

The workspace [`AGENTS.md`](../AGENTS.md) is a thin entry point that points here. Build/lint/
test commands, the crate layout, code style, the data pipeline, and the OpenAPI wiring each
have their own `design/` doc (see the table below).

---

## Design & architecture (`design/`)

| Doc | What it covers | Read when… |
|---|---|---|
| [`design/commands.md`](design/commands.md) | Build, lint, test, format, run-server, and data-conversion commands (all standard Cargo). | You need to build, run, test, format, lint, or regenerate the persistent data. |
| [`design/workspace-structure.md`](design/workspace-structure.md) | The crate & module map: what `spf` / `spf_core` / `spf_cli` / `spf_macros` each contain. | You are orienting yourself or looking for where a piece of logic lives. |
| [`design/data-pipeline.md`](design/data-pipeline.md) | Offline card ingestion vs. runtime load (PDF→txt→JSON→memory), the persistent format, and the serialization contract. | You are touching data loading, persistence, the `spf_cli` converter, or the card data. |
| [`design/code-style.md`](design/code-style.md) | Formatting, naming, imports, types, error handling, comments, and the key architectural patterns (`PlayImpl`, `lazy_static!` tables, derive macros, serde). | You are writing or reviewing code and want the project's conventions. |
| [`design/openapi-utoipa.md`](design/openapi-utoipa.md) | The dev environment (DevContainer, ports) and how the OpenAPI/Swagger spec is generated from code via utoipa. | You are working on HTTP handlers, the API spec, or setting up the dev environment. |
| [`design/testing-strategy.md`](design/testing-strategy.md) | The durable testing approach: philosophy, test layers, conventions, determinism (FAC deck seam), fixtures, current test inventory, and how to run tests. | You are writing or reviewing tests, or deciding *how* something should be tested. |
| [`design/ws-events-architecture.md`](design/ws-events-architecture.md) | The WebSocket/event-broadcast architecture: domain-vs-transport layering, the `tokio::sync::broadcast` bridge, `Game` as event emitter, transport adapters, and extensibility guarantees. | You are touching game events, adding a new event type, or adding a client-facing event transport. |

## Work plans (`plans/`)

| Doc | What it covers | Read when… |
|---|---|---|
| [`plans/testing-plan.md`](plans/testing-plan.md) | Staged roadmap (T1–T6) for building out the test suite, with status tracking and WS tie-ins. | You are picking up the next testing task or checking testing progress. |
| [`plans/tech-debt.md`](plans/tech-debt.md) | Running log of cross-cutting rough edges to revisit (e.g. CWD-relative resource paths). | You hit a general problem worth recording, or are picking one up to fix. |
| [`plans/ws-events-stages.md`](plans/ws-events-stages.md) | High-level staged rollout of the WebSocket events feature (Stages 1–4). **Stages 1–2 done; Stage 3 next.** | You are resuming the WS feature and need the stage overview. |
| [`plans/ws-events-stage1.md`](plans/ws-events-stage1.md) | Detailed, file-level task list for WS Stage 1 (dependencies + `GameEvent` type). **✅ Done** — retained as the record of what shipped. | You want the record of what WS Stage 1 delivered. |
| [`plans/ws-events-stage2.md`](plans/ws-events-stage2.md) | Detailed, file-level task list for WS Stage 2 (`Game` broadcast `Sender`, `emit()`/`subscribe()`, emission sites, unit test). **✅ Done.** | You want the record of what WS Stage 2 delivered. |

---

## Conventions for adding docs

- **New design/arch doc** → add to `design/` and link it in the table above.
- **New work plan** → add to `plans/` and link it in the table above.
- **Cross-links:** reference sibling docs by relative path
  (`../design/foo.md` from a plan, `../plans/bar.md` from a design doc).
- Keep this index current: every new doc under `docs/` should appear in one of the tables.

---

## Related top-level docs

| Doc | Location | Notes |
|---|---|---|
| `AGENTS.md` | workspace root | Thin entry point for agents: quick orientation + a table pointing into these docs. |
| `README.md` | workspace root | Project overview and API exploration entry points. |
| `Backlog.md` | workspace root | Game-rules backlog (features not yet implemented). |
