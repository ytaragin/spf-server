//! `spf_core` — the shared Statis Pro Football data model.
//!
//! This crate contains the player/stat data structures, the text-file loaders
//! that build them from the `pdftotext` output, and a persistent JSON format
//! (see [`persist`]). It is shared by the `spf` server (which loads the
//! persistent data at startup) and the `spf_cli` tool (which generates it).

pub mod lineup;
pub mod loader;
pub mod persist;
pub mod players;
pub mod shiftable;
pub mod stats;
