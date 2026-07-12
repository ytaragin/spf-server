# Completed Plans

An archive of work plans that have **fully shipped**. Each entry below is a high-level
topic; when a topic's last stage lands, its detailed per-stage files move here from
`../` (the active `plans/` directory) so that directory stays focused on what's still in
flight.

This file is the durable reference for "what did we plan, and what actually landed" for
completed topics. The top-level [`docs/README.md`](../../README.md) only links to this
folder and lists topics by name — it does not link the individual files below, since the
set of files per topic can vary (a stage overview + one doc per stage). Come here when you
need that level of detail.

---

## Completed topics

### Adding WebSockets to the server (live game events)

Real-time push of game state changes (lineup set, play run, game lifecycle, …) to
connected clients over a read-only WebSocket, built alongside the existing REST API. See
[`../../design/ws-events-architecture.md`](../../design/ws-events-architecture.md) for the
durable design (domain/transport layering, the broadcast-channel bridge) — this archive
covers *how the rollout actually happened*, stage by stage.

| File | What it covers |
|---|---|
| [`ws-events-stages.md`](ws-events-stages.md) | High-level staged rollout plan (4 stages) with a "what landed" summary per stage. Start here. |
| [`ws-events-stage1.md`](ws-events-stage1.md) | Stage 1 — foundation: crate dependencies (`actix-ws`, `tokio`, `futures-util`) and the `GameEvent` enum. |
| [`ws-events-stage2.md`](ws-events-stage2.md) | Stage 2 — domain emitter: `Game` gains a broadcast `Sender`, `emit()`/`subscribe()`, and emits at every state-mutating method. |
| [`ws-events-stage3.md`](ws-events-stage3.md) | Stage 3 — WebSocket transport: the `GET /game/ws` handler, snapshot-then-stream behavior, and the utoipa registration-order gotcha discovered along the way. |

Stage 4 (OpenAPI schema registration + `AGENTS.md` update) is recorded inline in
`ws-events-stages.md` rather than as its own file (it touched only the `ApiDoc` schema list
and doc files, with no new design decisions to detail).

Known follow-ups from this work are tracked as open items in
[`../tech-debt.md`](../tech-debt.md) (no automated WS transport test; no resync-after-lag;
the `utoipa`/`OpenApiFactory` registration gotcha needs a durable write-up in
`docs/design/openapi-utoipa.md`).

---

## Conventions

- Move a topic here **only when every stage is done** — a topic with any stage still
  `⬜ Not started` stays in `plans/`.
- Preserve the original per-stage files as-is (aside from fixing relative links broken by
  the move); they are the historical record of what was planned vs. what shipped.
- Add a new "Completed topics" entry (a short paragraph + a file table) per topic, not per
  file — the top-level `docs/README.md` links only this file, not individual entries here.
