//! `fs-surveys-server`: the root of the contract `surveys.capnp` over Cap'n Proto.
//!
//! Served: `supportReport` and `types` (the catalog of type descriptions for the generated frontend,
//! ADR 0066). Not served yet (the generated method answers "unimplemented", named in `DESIGN.md`):
//! `capabilities`, `designer`, `respond`, `results`, `decisions`, `subscribe`. The app talks to the core
//! in its own process; the other sub-interfaces are the next step.
//!
//! Interface status: draft (ADR 0045).
//!
//! Pattern: Facade — this file only declares modules and re-exports.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// The generated `Server` traits are async for every method, also those that never wait.
#![allow(clippy::missing_errors_doc, clippy::unused_async_trait_impl)]

mod root;

pub use root::SurveysServer;
