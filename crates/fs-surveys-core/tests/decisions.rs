//! Guarantee 7 (second part): decisions and their links to tasks.

mod common;

use common::*;
use fs_store::ObjectId;
use fs_surveys_core::{ALL_STAGES, Anonymity, Error, Method};

#[tokio::test]
async fn decisions_link_to_tasks_and_back() {
    let e = env().await;
    let (s, q) = open_survey(&e, Anonymity::Named).await;
    let d = e
        .repo
        .record(
            s.id(),
            q,
            Method::Majority,
            ALL_STAGES,
            vec!["b".into()],
            "we go with b",
        )
        .await
        .unwrap();
    assert_eq!(
        (d.outcome.clone(), d.stage, d.method),
        (vec!["b".to_string()], ALL_STAGES, Method::Majority)
    );
    for bad in [vec!["zzz".to_string()]] {
        let r = e
            .repo
            .record(s.id(), q, Method::Majority, ALL_STAGES, bad, "")
            .await;
        assert!(matches!(r, Err(Error::Invalid(_))));
    }
    let r = e
        .repo
        .record(
            s.id(),
            ObjectId::new(),
            Method::Majority,
            ALL_STAGES,
            vec![],
            "",
        )
        .await;
    assert!(matches!(r, Err(Error::NotFound { .. })));
    let t1 = "https://example.org/tasks/1";
    let t2 = "https://example.org/tasks/2";
    e.repo.link_task(d.id(), t1).await.unwrap();
    let linked = e.repo.link_task(d.id(), t1).await.unwrap();
    assert_eq!(linked.tasks, [t1], "linking twice is once");
    e.repo.link_task(d.id(), t2).await.unwrap();
    assert_eq!(e.repo.of_task(t1).await.unwrap().len(), 1);
    assert_eq!(e.repo.of_task(t1).await.unwrap()[0].id(), d.id());
    let unlinked = e.repo.unlink_task(d.id(), t1).await.unwrap();
    assert_eq!(unlinked.tasks, [t2]);
    assert!(e.repo.of_task(t1).await.unwrap().is_empty());
    assert_eq!(e.repo.of_task(t2).await.unwrap().len(), 1);
    assert_eq!(
        e.repo
            .list_decisions(s.id(), None)
            .await
            .unwrap()
            .items
            .len(),
        1
    );
    assert_eq!(
        e.repo.get_decision(d.id()).await.unwrap().unwrap().note,
        "we go with b"
    );
}
