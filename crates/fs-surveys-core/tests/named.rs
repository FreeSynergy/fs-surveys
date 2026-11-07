//! Guarantee 8: named answers, the feed, and a delete that leaves nothing.

mod common;

use std::sync::{Arc, Mutex};

use common::*;
use fs_objects::{Query, Store};
use fs_surveys_core::{
    ALL_STAGES, Anonymity, Credential, Error, SurveyChange, SurveyFeed, SurveyListener, SurveyState,
};

#[derive(Default)]
struct Seen(Mutex<Vec<SurveyChange>>);

impl SurveyListener for Seen {
    fn changed(&self, c: &SurveyChange) {
        self.0.lock().unwrap().push(c.clone());
    }
}

#[tokio::test]
async fn save_resume_submit_and_replace() {
    let e = env().await;
    let (s, q) = open_survey(&e, Anonymity::Named).await;
    let ada = person("ada");
    assert!(e.repo.resume(s.id(), &ada).await.unwrap().is_none());
    let seen = Arc::new(Seen::default());
    let _sub = e.repo.subscribe(seen.clone());
    let draft = e.repo.save_response(s.id(), &ada, vec![]).await.unwrap();
    assert!(!draft.submitted);
    let draft = e
        .repo
        .save_response(s.id(), &ada, choice(q, "a"))
        .await
        .unwrap();
    assert_eq!(
        e.repo.resume(s.id(), &ada).await.unwrap().unwrap().answers,
        choice(q, "a")
    );
    assert_eq!(
        e.repo.summary(s.id(), ALL_STAGES).await.unwrap().ballots,
        0,
        "a draft does not count"
    );
    assert!(
        seen.0.lock().unwrap().is_empty(),
        "saving a draft is not an answer"
    );
    e.repo
        .submit(s.id(), Credential::Person(ada.clone()), choice(q, "a"))
        .await
        .unwrap();
    e.repo
        .submit(s.id(), Credential::Person(ada.clone()), choice(q, "c"))
        .await
        .unwrap();
    let sum = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!(
        (sum.ballots, count(&sum, q, "a"), count(&sum, q, "c")),
        (1, 0, 1),
        "replaced, live, no minGroup"
    );
    let r = e.repo.resume(s.id(), &ada).await.unwrap().unwrap();
    assert!(
        r.submitted && r.envelope.id == draft.envelope.id,
        "one object per person and survey"
    );
    let events = seen.0.lock().unwrap().clone();
    assert_eq!(events.len(), 2);
    assert!(
        events
            .iter()
            .all(|c| matches!(c, SurveyChange::Answered(sid, _) if *sid == s.id()))
    );
    let again = e.repo.save_response(s.id(), &ada, choice(q, "b")).await;
    assert!(matches!(again, Err(Error::Refused(_))));
    let anon = open_survey(&e, Anonymity::AnonymousOpen).await.0;
    assert!(
        matches!(
            e.repo.save_response(anon.id(), &ada, vec![]).await,
            Err(Error::Refused(_))
        ),
        "named only"
    );
}

#[tokio::test]
async fn state_changes_reach_the_feed_until_unsubscribed() {
    let e = env().await;
    let seen = Arc::new(Seen::default());
    let sub = e.repo.subscribe(seen.clone());
    let (s, _) = open_survey(&e, Anonymity::Named).await;
    e.repo.close(s.id()).await.unwrap();
    assert!(
        seen.0
            .lock()
            .unwrap()
            .contains(&SurveyChange::State(s.id(), SurveyState::Closed))
    );
    drop(sub);
    let before = seen.0.lock().unwrap().len();
    e.repo.reopen(s.id()).await.unwrap();
    assert_eq!(seen.0.lock().unwrap().len(), before);
}

#[tokio::test]
async fn delete_leaves_nothing_behind() {
    let e = env().await;
    let (s, q) = open_survey(&e, Anonymity::Named).await;
    e.repo
        .submit(s.id(), Credential::Person(person("ada")), choice(q, "a"))
        .await
        .unwrap();
    e.repo.invite(s.id(), &[person("bob")], &[]).await.unwrap();
    e.repo
        .record(
            s.id(),
            q,
            fs_surveys_core::Method::Plurality,
            ALL_STAGES,
            vec!["a".into()],
            "go",
        )
        .await
        .unwrap();
    assert_eq!(e.main.query(&Query::new()).await.unwrap().len(), 4);
    e.repo.delete(s.id()).await.unwrap();
    assert!(e.main.query(&Query::new()).await.unwrap().is_empty());
    assert!(e.repo.get(s.id()).await.unwrap().is_none());

    let (a, q) = open_survey(&e, Anonymity::AnonymousOnce).await;
    e.repo.invite(a.id(), &[person("ada")], &[]).await.unwrap();
    e.repo
        .submit(
            a.id(),
            Credential::Token(e.delivery.token_of(&person("ada"))),
            choice(q, "a"),
        )
        .await
        .unwrap();
    assert!(e.ballots.query(&Query::new()).await.unwrap().len() == 1);
    e.repo.delete(a.id()).await.unwrap();
    assert!(
        e.ballots.query(&Query::new()).await.unwrap().is_empty(),
        "the box is gone"
    );
    assert!(e.main.query(&Query::new()).await.unwrap().is_empty());
    assert!(matches!(
        e.repo.delete(a.id()).await,
        Err(Error::NotFound { .. })
    ));
}
