//! Review: one lock per survey, from read to save, shared by clones.

mod common;

use common::*;
use fs_store::traits::UnloggedObjects;
use fs_surveys_core::{ALL_STAGES, Anonymity, Credential, SurveyState};

const ROUNDS: usize = 30;

macro_rules! par {
    ($a:expr, $b:expr) => {{
        let (h1, h2) = (tokio::spawn($a), tokio::spawn($b));
        (h1.await.unwrap(), h2.await.unwrap())
    }};
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn publish_and_edit_never_bring_a_survey_back_to_draft() {
    for _ in 0..ROUNDS {
        let e = env().await;
        let s = e
            .repo
            .create("S", Anonymity::AnonymousOpen, &[])
            .await
            .unwrap();
        let id = s.id();
        e.repo.add_question(id, pick()).await.unwrap();
        let (r1, r2) = (e.repo.clone(), e.repo.clone());
        let (p, ed) = par!(async move { r1.publish(id).await.is_ok() }, async move {
            r2.edit(id, "S", Anonymity::Named, 5).await.is_ok()
        });
        let now = e.repo.get(id).await.unwrap().unwrap();
        // Either order is fine; both succeeding means publish came first, so edit must have failed.
        assert!(!(p && ed), "edit of a published survey went through");
        assert_eq!(now.state == SurveyState::Open, p);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn parallel_add_question_and_close_lose_nothing() {
    for _ in 0..ROUNDS {
        let e = env().await;
        let s = e
            .repo
            .create("S", Anonymity::AnonymousOpen, &[])
            .await
            .unwrap();
        let id = s.id();
        let (r1, r2) = (e.repo.clone(), e.repo.clone());
        let (a, b) = par!(
            async move { r1.add_question(id, pick()).await.is_ok() },
            async move { r2.add_question(id, pick()).await.is_ok() }
        );
        assert!(a && b);
        assert_eq!(e.repo.get(id).await.unwrap().unwrap().questions.len(), 2);
        e.repo.publish(id).await.unwrap();
        let (r1, r2) = (e.repo.clone(), e.repo.clone());
        let _ = par!(
            async move { r1.classify(id, &[]).await.is_ok() },
            async move { r2.close(id).await.is_ok() }
        );
        assert_eq!(
            e.repo.get(id).await.unwrap().unwrap().state,
            SurveyState::Closed
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn parallel_invites_send_one_token() {
    for _ in 0..ROUNDS {
        let e = env().await;
        let (s, _) = open_survey(&e, Anonymity::AnonymousOnce).await;
        let p = vec![person("ada")];
        let (r1, r2, p1, p2) = (e.repo.clone(), e.repo.clone(), p.clone(), p.clone());
        let id = s.id();
        let _ = par!(
            async move { r1.invite(id, &p1, &[]).await.unwrap() },
            async move { r2.invite(id, &p2, &[]).await.unwrap() }
        );
        assert_eq!(e.delivery.all().len(), 1);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn parallel_named_submits_make_one_response() {
    for _ in 0..ROUNDS {
        let e = env().await;
        let (s, q) = open_survey(&e, Anonymity::Named).await;
        let id = s.id();
        let (r1, r2) = (e.repo.clone(), e.repo.clone());
        let who = person("ada");
        let (w1, w2) = (who.clone(), who);
        let _ = par!(
            async move {
                r1.submit(id, Credential::Person(w1), choice(q, "a"))
                    .await
                    .unwrap()
            },
            async move {
                r2.submit(id, Credential::Person(w2), choice(q, "b"))
                    .await
                    .unwrap()
            }
        );
        assert_eq!(e.repo.summary(id, ALL_STAGES).await.unwrap().ballots, 1);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn a_submit_cannot_recreate_the_box_of_a_deleted_survey() {
    for _ in 0..ROUNDS {
        let e = env().await;
        let (s, q) = open_survey(&e, Anonymity::AnonymousOpen).await;
        let id = s.id();
        let (r1, r2) = (e.repo.clone(), e.repo.clone());
        let (_, deleted) = par!(
            async move {
                r1.submit(id, Credential::None, choice(q, "a"))
                    .await
                    .is_ok()
            },
            async move { r2.delete(id).await.is_ok() }
        );
        assert!(deleted);
        assert!(
            e.ballots.read_unlogged(id).await.unwrap().is_none(),
            "orphan box"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn a_submit_cannot_land_after_the_close() {
    for _ in 0..ROUNDS {
        let e = env().await;
        let s = e
            .repo
            .create("S", Anonymity::AnonymousOpen, &[])
            .await
            .unwrap();
        e.repo
            .edit(s.id(), "S", Anonymity::AnonymousOpen, 3)
            .await
            .unwrap();
        let (_, q) = e.repo.add_question(s.id(), pick()).await.unwrap();
        e.repo.publish(s.id()).await.unwrap();
        submit_open(&e, &s, q, "a", 3).await;
        let id = s.id();
        let (r1, r2) = (e.repo.clone(), e.repo.clone());
        let (landed, _) = par!(
            async move {
                r1.submit(id, Credential::None, choice(q, "a"))
                    .await
                    .is_ok()
            },
            async move { r2.close(id).await.is_ok() }
        );
        let sum = e.repo.summary(id, ALL_STAGES).await.unwrap();
        assert_eq!(sum.held_back, 0, "nothing is left pending after the close");
        assert_eq!(sum.ballots, if landed { 4 } else { 3 });
    }
}
