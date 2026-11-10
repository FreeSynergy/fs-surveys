//! `Surveys.types` and `Surveys.supportReport` over a real socket and a real store.

use std::sync::Arc;

use fs_interfaces::rpc::{bind_socket, connect, serve};
use fs_interfaces::surveys_capnp::surveys;
use fs_objects::TursoStore;
use fs_surveys_core::{SurveyRepo, commands, type_registry};
use fs_surveys_server::SurveysServer;
use fs_types::TypeRegistry;

async fn with_client<F, Fut>(test: F)
where
    F: FnOnce(surveys::Client) -> Fut,
    Fut: Future<Output = ()>,
{
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let main = Arc::new(TursoStore::open_in_memory().await.unwrap());
            let ballots = Arc::new(TursoStore::open_in_memory().await.unwrap());
            let repo = Arc::new(SurveyRepo::new(
                main,
                ballots,
                "https://example.org/users/owner",
            ));
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("surveys.sock");
            let bound = bind_socket(&path).await.unwrap();
            let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
            let root = SurveysServer::new(repo).into_client();
            let serving = tokio::task::spawn_local(serve(bound, root.client, async {
                let _ = stopped.await;
            }));
            let connection = connect::<surveys::Client>(&path).await.unwrap();
            test(connection.client.clone()).await;
            drop(connection);
            let _ = stop.send(());
            serving.await.unwrap().unwrap();
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn types_serves_the_whole_catalog_with_the_flag() {
    with_client(|root| async move {
        let catalog = root.types_request().send().promise.await.unwrap();
        let catalog = catalog.get().unwrap().get_catalog().unwrap();
        let answer = catalog.describe_request().send().promise.await.unwrap();
        let read = TypeRegistry::read(answer.get().unwrap().get_types().unwrap()).unwrap();
        assert_eq!(read.len(), 4);
        let sent: usize = read.iter().map(|t| t.commands.len()).sum();
        assert_eq!(sent, commands::all().len());
        for name in ["Survey", "Question", "SurveyResponse", "Decision"] {
            assert_eq!(read.get(name), type_registry().unwrap().get(name), "{name}");
        }
        let survey = read.get("Survey").unwrap();
        assert_eq!(survey.program, "fs-surveys");
        let public = survey.find_command("publicLink").unwrap();
        assert!(
            !public.supported,
            "unsupported commands are described, and say so"
        );
        assert!(survey.find_command("create").unwrap().supported);
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn the_support_report_names_what_is_not_built() {
    with_client(|root| async move {
        let answer = root.support_report_request().send().promise.await.unwrap();
        let report = answer.get().unwrap().get_report().unwrap();
        let entries = report.get_entries().unwrap();
        assert!(entries.len() > 20);
    })
    .await;
}
