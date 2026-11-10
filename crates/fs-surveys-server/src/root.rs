//! The `Surveys` root capability.
//!
//! Pattern: Adapter — the generated `surveys::Server` trait over the traits of the core. It depends on
//! `fs_surveys_core::Surveys`, never on `SurveyRepo` (ADR 0043). `types` hands out the catalog of the four
//! data types with every command, supported or not.

use std::rc::Rc;
use std::sync::Arc;

use capnp_rpc::new_client;
use fs_interfaces::surveys_capnp::surveys;
use fs_surveys_core::{Surveys, type_registry};
use fs_types::describe::CatalogServer;

/// The version named in the header of `surveys.capnp`.
const CONTRACT_VERSION: &str = "surveys 0.1 (draft)";

type Done = Result<(), capnp::Error>;

/// The root capability of the surveys program.
pub struct SurveysServer {
    surveys: Arc<dyn Surveys>,
}

impl SurveysServer {
    /// Serve this provider.
    pub fn new<P: Surveys + 'static>(provider: Arc<P>) -> Self {
        Self { surveys: provider }
    }

    /// The capability to hand to `fs_interfaces::rpc::serve`.
    #[must_use]
    pub fn into_client(self) -> surveys::Client {
        new_client(self)
    }
}

impl surveys::Server for SurveysServer {
    async fn support_report(
        self: Rc<Self>,
        _: surveys::SupportReportParams,
        mut results: surveys::SupportReportResults,
    ) -> Done {
        self.surveys
            .capabilities()
            .write(CONTRACT_VERSION, results.get().init_report());
        Ok(())
    }

    async fn types(
        self: Rc<Self>,
        _: surveys::TypesParams,
        mut results: surveys::TypesResults,
    ) -> Done {
        // The data types of this program and every command, also the ones it cannot run yet (ADR 0066).
        let registry = type_registry().map_err(|e| capnp::Error::failed(e.to_string()))?;
        results
            .get()
            .set_catalog(new_client(CatalogServer::new(registry)));
        Ok(())
    }
}
