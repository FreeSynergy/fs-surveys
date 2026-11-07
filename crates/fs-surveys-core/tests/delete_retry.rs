//! Review: delete removes the box first, can be repeated, and a late submit recreates nothing.

mod common;

use common::*;
use fs_store::traits::UnloggedObjects;
use fs_surveys_core::{Anonymity, Error};

#[tokio::test]
async fn delete_removes_the_box_first_and_can_be_repeated() {
    let e = env().await;
    let (s, q) = open_survey(&e, Anonymity::AnonymousOpen).await;
    submit_open(&e, &s, q, "a", 1).await;
    assert!(e.ballots.read_unlogged(s.id()).await.unwrap().is_some());
    e.repo.delete(s.id()).await.unwrap();
    assert!(e.ballots.read_unlogged(s.id()).await.unwrap().is_none());
    let late = e.repo.submit(s.id(), human(), choice(q, "a")).await;
    assert!(matches!(late, Err(Error::NotFound { .. })));
    assert!(
        e.ballots.read_unlogged(s.id()).await.unwrap().is_none(),
        "no box is recreated"
    );
}
