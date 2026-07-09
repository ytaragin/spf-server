# Data Pipeline: Offline Ingestion vs. Runtime

Durable reference for how raw player-card data becomes in-memory game state. Card ingestion
is deliberately **split** from the server: heavy parsing happens offline via a CLI tool, and
the server only loads a pre-built persistent format at startup.

For the crates involved, see [`workspace-structure.md`](workspace-structure.md).

---

## The three stages

1. **PDF → txt** (unchanged): `pdftodat.zsh` shells out to `pdftotext` to produce the `.txt`
   files in `cards/SPFB1983/`.
2. **txt → JSON** (the `spf_cli` tool): `spf-cli convert --cards-dir cards/SPFB1983 --year 1983`
   parses the `.txt` files via `spf_core::loader` and writes the persistent model to `data/1983/`
   (one `<TeamName>.json` per team + an `index.json` manifest). `data/` is git-ignored — it is a
   locally-generated build artifact, not committed.
3. **JSON → memory** (the server): `main.rs` calls `spf_core::persist::load_league("data/1983")`
   at startup (hardcoded path). If the manifest is missing the server exits with a clear error;
   it no longer parses `.txt` files at runtime.

---

## Persistent format & serialization contract

The persistent format stores each roster's players as a JSON array of the internally-tagged
`Player` enum (`{ "QB": { … } }`). On load, `Roster::from_players` maps each variant back into a
`Box<dyn BasePlayer>`. Because trait objects cannot be auto-deserialized, **every** `*Stats` struct
and stat helper (`RangedStats`, `Returner`, etc.) derives both `Serialize` and `Deserialize`; keep
it that way when adding fields/types.

> **FAC deck note:** `fac_cards.csv` is still parsed at runtime (lazily, per-game) by
> `game/fac.rs` — it was intentionally left out of the persistent format for now, so the `cards/`
> directory must remain present at runtime.

---

## Regenerating the persistent data

```
cargo run -p spf_cli -- convert --cards-dir cards/SPFB1983 --year 1983
```

See [`commands.md`](commands.md) for the full command reference.

---

## Where the data lives at runtime

Raw player card data lives in `cards/SPFB1983/` (text files) and `cards/fac_cards.csv`. The
`.txt` files are converted **offline** into `data/1983/*.json` by the `spf_cli` tool and loaded
at startup via `spf_core::persist::load_league`. `fac_cards.csv` is still parsed at runtime by
`game/fac.rs`.
