//! What every backend shares: the core, a runtime to wait for it, the instance for IRIs.
//!
//! Pattern: Adapter (sync to async) — `Backend` is synchronous (the frontend calls it with its session
//! locked, `fs-admin-frontend` DESIGN.md), the core is async; `block` is the one place that waits.

use std::path::Path;
use std::sync::Arc;

use fs_objects::{Store, TursoStore, traits::UnloggedObjects};
use fs_surveys_core::SurveyRepo;
use tokio::runtime::Runtime;

use crate::operator::OperatorIsHuman;
use crate::refuse::RunError;

/// The shared parts of the four backends.
pub struct Ctx {
    /// The core.
    pub repo: SurveyRepo,
    /// Domain of the IRIs this program makes (ADR 0027).
    pub instance: String,
    rt: Runtime,
}

impl Ctx {
    /// Wait for a core call. Never call this from inside an async task.
    pub fn block<F: std::future::Future>(&self, f: F) -> F::Output {
        self.rt.block_on(f)
    }

    /// Stores in memory (tests, a first look).
    ///
    /// # Errors
    /// [`RunError::Store`] if a store cannot be opened.
    pub fn in_memory(actor: &str, instance: &str) -> Result<Arc<Self>, RunError> {
        let rt = runtime()?;
        let repo = rt.block_on(async {
            let main: Arc<dyn Store> = Arc::new(TursoStore::open_in_memory().await?);
            let ballots: Arc<dyn UnloggedObjects> = Arc::new(TursoStore::open_in_memory().await?);
            Ok::<_, fs_objects::Error>(
                SurveyRepo::new(main, ballots, actor)
                    .with_instance(instance)
                    .with_ai_marking(Arc::new(OperatorIsHuman::new(actor))),
            )
        });
        Ok(Arc::new(Self {
            repo: repo.map_err(|e| RunError::Store(e.to_string()))?,
            instance: instance.to_owned(),
            rt,
        }))
    }

    /// The two stores in `dir`: `objects.db` (surveys, named answers, decisions) and `ballots.db`
    /// (anonymous ballots, a store of its own that is never connected to the hub).
    ///
    /// # Errors
    /// [`RunError::Store`] if a store cannot be opened.
    pub fn open(dir: &Path, actor: &str, instance: &str) -> Result<Arc<Self>, RunError> {
        std::fs::create_dir_all(dir).map_err(|e| RunError::Store(e.to_string()))?;
        let rt = runtime()?;
        let repo = rt.block_on(async {
            let main: Arc<dyn Store> = Arc::new(TursoStore::open(dir.join("objects.db")).await?);
            let ballots: Arc<dyn UnloggedObjects> =
                Arc::new(TursoStore::open(dir.join("ballots.db")).await?);
            Ok::<_, fs_objects::Error>(
                SurveyRepo::new(main, ballots, actor)
                    .with_instance(instance)
                    .with_ai_marking(Arc::new(OperatorIsHuman::new(actor))),
            )
        });
        Ok(Arc::new(Self {
            repo: repo.map_err(|e| RunError::Store(e.to_string()))?,
            instance: instance.to_owned(),
            rt,
        }))
    }
}

fn runtime() -> Result<Runtime, RunError> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| RunError::Store(e.to_string()))
}
