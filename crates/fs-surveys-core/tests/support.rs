//! The provider rule (ADR 0045): capabilities, `Unsupported` defaults, what is not built.

mod common;

use async_trait::async_trait;
use common::*;
use fs_store::ObjectId;
use fs_surveys_core::{
    Anonymity, Credential, Cursor, Decisions, Designer, Error, Page, Respond, Result, Survey,
    Surveys,
};

const NAMED_IN_CONTRACT: [&str; 26] = [
    "survey.create",
    "survey.edit",
    "survey.anonymity",
    "survey.classify",
    "question.add",
    "question.edit",
    "question.move",
    "question.validate",
    "logic.set",
    "survey.publish",
    "survey.close",
    "survey.invite",
    "survey.tokens",
    "survey.widen",
    "survey.open",
    "response.save",
    "response.submit",
    "survey.publicLink",
    "result.summary",
    "survey.method",
    "result.export",
    "result.filter",
    "decision.record",
    "decision.read",
    "decision.link",
    "survey.subscribe",
];

#[tokio::test]
async fn every_capability_of_the_contract_is_listed() {
    let e = env().await;
    let caps = e.repo.capabilities();
    for id in NAMED_IN_CONTRACT.iter().chain(&["poll.embed"]) {
        assert!(
            caps.entries().iter().any(|s| s.capability == *id),
            "{id} missing"
        );
    }
    let why = caps.why_not("survey.publicLink").expect("not built");
    assert!(!why.reason().is_empty());
    assert!(caps.why_not("poll.embed").is_some());
    assert!(caps.is_supported("survey.widen") && caps.is_supported("result.filter"));
    assert!(
        caps.entries()
            .iter()
            .filter(|s| !s.supported)
            .all(|s| !s.reason.is_empty())
    );
}

#[tokio::test]
async fn public_surveys_answer_unsupported() {
    let e = env().await;
    let s = e.repo.create("Open", Anonymity::Public, &[]).await.unwrap();
    let (_, q) = e.repo.add_question(s.id(), pick()).await.unwrap();
    let r = e.repo.publish(s.id()).await;
    assert!(
        matches!(&r, Err(Error::Unsupported(u)) if u.capability() == "survey.publicLink"),
        "{r:?}"
    );
    for r in [
        e.repo
            .submit(s.id(), Credential::Proof(vec![1]), choice(q, "a"))
            .await,
        e.repo
            .submit(s.id(), Credential::None, choice(q, "a"))
            .await,
    ] {
        assert!(
            matches!(&r, Err(Error::Unsupported(u)) if u.capability() == "survey.publicLink"),
            "{r:?}"
        );
    }
    let r = Respond::challenge(&e.repo, s.id()).await;
    assert!(matches!(r, Err(Error::Unsupported(_))));
    assert!(matches!(
        e.repo.invite(s.id(), &[person("a")], &[]).await,
        Err(Error::Unsupported(_))
    ));
}

struct ReadOnly;

#[async_trait]
impl Designer for ReadOnly {
    async fn get(&self, _id: ObjectId) -> Result<Option<Survey>> {
        Ok(None)
    }
    async fn list(&self, _cursor: Option<&Cursor>) -> Result<Page<Survey>> {
        Ok(Page {
            items: vec![],
            next: None,
        })
    }
}

#[async_trait]
impl Decisions for ReadOnly {
    async fn get_decision(&self, _id: ObjectId) -> Result<Option<fs_surveys_core::Decision>> {
        Ok(None)
    }
    async fn list_decisions(
        &self,
        _s: ObjectId,
        _c: Option<&Cursor>,
    ) -> Result<Page<fs_surveys_core::Decision>> {
        Ok(Page {
            items: vec![],
            next: None,
        })
    }
    async fn of_task(&self, _task: &str) -> Result<Vec<fs_surveys_core::Decision>> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn a_provider_that_overrides_only_reads_says_unsupported_for_the_rest() {
    let p = ReadOnly;
    let id = ObjectId::new();
    let unsupported = |r: Result<()>, cap: &str| match r {
        Err(Error::Unsupported(u)) => assert_eq!(u.capability(), cap),
        other => panic!("{other:?}"),
    };
    unsupported(
        p.create("x", Anonymity::Named, &[]).await.map(drop),
        "survey.create",
    );
    unsupported(p.publish(id).await.map(drop), "survey.publish");
    unsupported(p.close(id).await.map(drop), "survey.close");
    unsupported(p.delete(id).await, "survey.create");
    unsupported(p.widen(id, id).await.map(drop), "survey.widen");
    unsupported(p.invite(id, &[], &[]).await.map(drop), "survey.invite");
    unsupported(p.link_task(id, "t").await.map(drop), "decision.link");
}

#[tokio::test]
async fn paging_follows_the_cursor() {
    let e = env().await;
    let repo = e.repo.clone().with_page_size(2);
    for n in ["a", "b", "c"] {
        repo.create(n, Anonymity::Named, &[]).await.unwrap();
    }
    let first = repo.list(None).await.unwrap();
    assert_eq!(first.items.len(), 2);
    let second = repo.list(first.next.as_ref()).await.unwrap();
    assert_eq!((second.items.len(), second.next), (1, None));
}
