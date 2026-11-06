//! Guarantee 5: widening over a real group hierarchy.

mod common;

use common::*;
use fs_store::ObjectId;
use fs_surveys_core::{ALL_STAGES, Anonymity, Error, Survey};

fn id(g: fs_store_02::ObjectId) -> ObjectId {
    g.to_string().parse().unwrap()
}

struct Tree {
    root: ObjectId,
    team: ObjectId,
    other: ObjectId,
}

async fn tree(e: &Env) -> Tree {
    let root = e.groups.create("Association", "org").await.unwrap();
    let team = e.groups.create("Team", "team").await.unwrap();
    let other = e.groups.create("Elsewhere", "org").await.unwrap();
    e.groups.add_parent(team.id, root.id).await.unwrap();
    Tree {
        root: id(root.id),
        team: id(team.id),
        other: id(other.id),
    }
}

async fn bound(e: &Env, scope: ObjectId) -> (Survey, ObjectId) {
    let s = e
        .repo
        .create("Budget", Anonymity::AnonymousOpen, &[scope])
        .await
        .unwrap();
    e.repo
        .edit(s.id(), "Budget", Anonymity::AnonymousOpen, 3)
        .await
        .unwrap();
    let (_, q) = e.repo.add_question(s.id(), pick()).await.unwrap();
    (e.repo.publish(s.id()).await.unwrap(), q)
}

#[tokio::test]
async fn results_stay_by_stage_and_anonymity_stays() {
    let e = env().await;
    let t = tree(&e).await;
    let (s, q) = bound(&e, t.team).await;
    submit_open(&e, &s, q, "a", 3).await;
    let wide = e.repo.widen(s.id(), t.root).await.unwrap();
    assert_eq!(
        (wide.stage, wide.scope.clone(), wide.anonymity),
        (1, vec![t.team, t.root], Anonymity::AnonymousOpen)
    );
    assert_eq!(wide.min_group, 3);
    submit_open(&e, &s, q, "b", 3).await;
    e.repo.close(s.id()).await.unwrap();
    let st0 = e.repo.summary(s.id(), 0).await.unwrap();
    let st1 = e.repo.summary(s.id(), 1).await.unwrap();
    let all = e.repo.summary(s.id(), ALL_STAGES).await.unwrap();
    assert_eq!((count(&st0, q, "a"), count(&st0, q, "b")), (3, 0));
    assert_eq!((count(&st1, q, "a"), count(&st1, q, "b")), (0, 3));
    assert_eq!(all.ballots, 6);
    assert!(e.repo.summary(s.id(), 2).await.unwrap().hidden);
}

#[tokio::test]
async fn a_small_rest_waits_for_the_next_release() {
    let e = env().await;
    let t = tree(&e).await;
    let (s, q) = bound(&e, t.team).await;
    submit_open(&e, &s, q, "a", 2).await;
    e.repo.widen(s.id(), t.root).await.unwrap();
    let sum = e.repo.summary(s.id(), 0).await.unwrap();
    assert_eq!(
        (sum.ballots, sum.held_back),
        (0, 0),
        "2 < minGroup: nothing released at stage 0, and the open survey shows no held-back number"
    );
    submit_open(&e, &s, q, "a", 1).await;
    e.repo.close(s.id()).await.unwrap();
    assert_eq!(e.repo.summary(s.id(), 1).await.unwrap().ballots, 3);
}

#[tokio::test]
async fn only_an_ancestor_of_the_scope_is_accepted() {
    let e = env().await;
    let t = tree(&e).await;
    let (s, _) = bound(&e, t.team).await;
    for parent in [t.other, t.team, ObjectId::new()] {
        let r = e.repo.widen(s.id(), parent).await;
        assert!(matches!(r, Err(Error::Refused(_))), "{r:?}");
    }
    let unbound = e.repo.create("Free", Anonymity::Named, &[]).await.unwrap();
    assert!(
        matches!(
            e.repo.widen(unbound.id(), t.root).await,
            Err(Error::Refused(_))
        ),
        "not open"
    );
    let (open, _) = open_survey(&e, Anonymity::Named).await;
    assert!(
        matches!(
            e.repo.widen(open.id(), t.root).await,
            Err(Error::Refused(_))
        ),
        "not bound"
    );
    e.repo.close(s.id()).await.unwrap();
    assert!(
        matches!(e.repo.widen(s.id(), t.root).await, Err(Error::Refused(_))),
        "closed"
    );
    assert_eq!(e.repo.get(s.id()).await.unwrap().unwrap().stage, 0);
}
