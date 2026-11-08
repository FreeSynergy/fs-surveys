//! Sensitive surveys: DPV categories, and the store that refuses them unencrypted.

mod common;

use std::sync::Arc;

use common::*;
use fs_objects::traits::UnloggedObjects;
use fs_objects::{OpenOptions, TursoStore};
use fs_surveys_core::{Anonymity, Error, SurveyRepo};

async fn file_store(opts: OpenOptions) -> (Arc<TursoStore>, std::path::PathBuf) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("fs-surveys-test-{}-{nanos}.db", std::process::id()));
    (
        Arc::new(TursoStore::open_with(&path, opts).await.unwrap()),
        path,
    )
}

fn cleanup(path: &std::path::Path) {
    for ext in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{ext}", path.display()));
    }
}

const HEALTH: &str = "pd:Health";

#[tokio::test]
async fn an_unencrypted_store_refuses_the_categories() {
    let (main, path) = file_store(OpenOptions::default()).await;
    let ballots = Arc::new(TursoStore::open_in_memory().await.unwrap());
    let repo = SurveyRepo::new(main, ballots, ME);
    let s = repo
        .create("Health check", Anonymity::AnonymousOpen, &[])
        .await
        .unwrap();
    let err = repo.classify(s.id(), &[HEALTH.into()]).await.unwrap_err();
    assert!(
        matches!(err, Error::Store(fs_objects::Error::EncryptionRequired(_))),
        "{err}"
    );
    assert!(
        repo.get(s.id())
            .await
            .unwrap()
            .unwrap()
            .envelope
            .sensitive
            .is_empty()
    );
    drop(repo);
    cleanup(&path);
}

#[tokio::test]
async fn opted_out_the_survey_and_the_box_carry_the_categories() {
    let opts = OpenOptions {
        allow_sensitive_unencrypted: true,
        ..OpenOptions::default()
    };
    let (main, path) = file_store(opts).await;
    let ballots = Arc::new(TursoStore::open_in_memory().await.unwrap());
    let repo = SurveyRepo::new(main, ballots.clone(), ME);
    let s = repo
        .create("Health check", Anonymity::AnonymousOpen, &[])
        .await
        .unwrap();
    let (_, q) = repo.add_question(s.id(), pick()).await.unwrap();
    let s = repo.classify(s.id(), &[HEALTH.into()]).await.unwrap();
    assert_eq!(s.envelope.sensitive, [HEALTH]);
    let s = repo.publish(s.id()).await.unwrap();
    repo.submit(s.id(), fs_surveys_core::Credential::None, choice(q, "a"))
        .await
        .unwrap();
    let b = ballots.read_unlogged(s.id()).await.unwrap().unwrap();
    assert_eq!(
        b.envelope.sensitive,
        [HEALTH],
        "the box copies the categories"
    );
    let s = repo
        .classify(s.id(), &["pd:Religion".into()])
        .await
        .unwrap();
    let b = ballots.read_unlogged(s.id()).await.unwrap().unwrap();
    assert_eq!(
        b.envelope.sensitive,
        ["pd:Religion"],
        "reclassifying reaches the box too"
    );
    drop(repo);
    cleanup(&path);
}

#[tokio::test]
async fn an_unencrypted_ballot_store_refuses_the_box_of_a_sensitive_survey() {
    let main = Arc::new(TursoStore::open_in_memory().await.unwrap());
    let (ballots, path) = file_store(OpenOptions::default()).await;
    let repo = SurveyRepo::new(main, ballots, ME);
    let s = repo
        .create("Faith", Anonymity::AnonymousOpen, &[])
        .await
        .unwrap();
    let (_, q) = repo.add_question(s.id(), pick()).await.unwrap();
    repo.classify(s.id(), &["pd:Religion".into()])
        .await
        .unwrap();
    let s = repo.publish(s.id()).await.unwrap();
    let r = repo
        .submit(s.id(), fs_surveys_core::Credential::None, choice(q, "a"))
        .await;
    assert!(
        matches!(
            r,
            Err(Error::Store(fs_objects::Error::EncryptionRequired(_)))
        ),
        "{r:?}"
    );
    drop(repo);
    cleanup(&path);
}
