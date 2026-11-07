//! Guarantee 3: tokens spend once, look alike when refused, and are never kept or returned.

mod common;

use std::sync::Arc;

use common::*;
use fs_surveys_core::{Anonymity, Credential, Error, SurveyRepo, Surveys};

async fn once(e: &Env) -> (fs_surveys_core::Survey, fs_objects::ObjectId) {
    open_survey(e, Anonymity::AnonymousOnce).await
}

#[tokio::test]
async fn a_token_spends_once_and_refusals_look_alike() {
    let e = env().await;
    let (s, q) = once(&e).await;
    let report = e
        .repo
        .invite(s.id(), &[person("ada"), person("bob")], &[])
        .await
        .unwrap();
    assert_eq!(report.invited.len(), 2);
    let t = e.delivery.token_of(&person("ada"));
    e.repo
        .submit(s.id(), Credential::Token(t.clone()), choice(q, "a"))
        .await
        .unwrap();
    let spent = e
        .repo
        .submit(s.id(), Credential::Token(t), choice(q, "a"))
        .await
        .unwrap_err();
    let made_up = e
        .repo
        .submit(s.id(), Credential::Token("0".repeat(64)), choice(q, "a"))
        .await
        .unwrap_err();
    assert!(matches!(spent, Error::InvalidToken));
    assert_eq!(spent.to_string(), made_up.to_string());
    assert_eq!(format!("{spent:?}"), format!("{made_up:?}"));
}

#[tokio::test]
async fn no_token_is_in_any_store_report_or_return_value() {
    let e = env().await;
    let (s, q) = once(&e).await;
    let report = e.repo.invite(s.id(), &[person("ada")], &[]).await.unwrap();
    let t = e.delivery.token_of(&person("ada"));
    assert_eq!(t.len(), 64);
    assert!(!format!("{report:?}").contains(&t));
    for store in [&e.main, &e.ballots] {
        assert!(
            !raw(store).await.contains(&t),
            "the token itself is kept nowhere"
        );
    }
    assert!(
        raw(&e.ballots).await.contains(&fs_hash(&t)),
        "only its hash is kept"
    );
    e.repo
        .submit(s.id(), Credential::Token(t.clone()), choice(q, "b"))
        .await
        .unwrap();
    assert!(
        !raw(&e.ballots).await.contains(&fs_hash(&t)),
        "spending removes the hash"
    );
    assert!(!format!("{:?}", Credential::Token(t.clone())).contains(&t));
    let again = e.repo.invite(s.id(), &[person("ada")], &[]).await.unwrap();
    assert_eq!(
        (again.invited.len(), again.already),
        (0, vec![person("ada")])
    );
    assert_eq!(e.delivery.all().len(), 1, "no second token");
    assert!(
        raw(&e.main).await.contains("people/ada"),
        "invited people are kept in the main store"
    );
    assert!(
        !raw(&e.ballots).await.contains("people/ada"),
        "...and not in the box"
    );
}

fn fs_hash(token: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[tokio::test]
async fn a_failed_delivery_leaves_no_usable_token() {
    let e = env().await;
    let repo = SurveyRepo::new(e.main.clone(), e.ballots.clone(), ME)
        .with_ai_marking(std::sync::Arc::new(common::ai::Marks))
        .with_delivery(Arc::new(Failing));
    let (s, q) = once(&e).await;
    let err = repo
        .invite(s.id(), &[person("ada")], &[])
        .await
        .unwrap_err();
    assert!(
        matches!(&err, Error::Delivery { person: p, .. } if *p == person("ada")),
        "{err}"
    );
    let tokens = e.repo.invite(s.id(), &[person("ada")], &[]).await.unwrap();
    assert_eq!(
        tokens.invited,
        [person("ada")],
        "a failed person did not count as invited"
    );
    let content = box_json(&e, &s).await;
    assert_eq!(
        content["people"]["tokens"].as_array().unwrap().len(),
        1,
        "only the delivered token has a hash"
    );
    let _ = q;
}

#[tokio::test]
async fn tokens_can_be_asked_for_only_with_a_delivery() {
    let e = env().await;
    let bare = SurveyRepo::new(e.main.clone(), e.ballots.clone(), ME);
    let (s, _) = once(&e).await;
    let err = bare
        .invite(s.id(), &[person("ada")], &[])
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));
    assert!(!bare.capabilities().is_supported("survey.tokens"));
    assert!(e.repo.capabilities().is_supported("survey.tokens"));
}

#[tokio::test]
async fn groups_are_invited_through_the_lookup() {
    let e = env().await;
    let g = e.groups.create("Team", "team").await.unwrap();
    for n in ["ada", "bob"] {
        e.groups
            .add(g.id, fs_groups_core::Member::Person(person(n)), "member")
            .await
            .unwrap();
    }
    let (s, _) = once(&e).await;
    let gid = g.id.to_string().parse().unwrap();
    let r = e
        .repo
        .invite(s.id(), &[person("ada")], &[gid])
        .await
        .unwrap();
    assert_eq!(r.invited.len(), 2);
    assert_eq!(e.delivery.all().len(), 2);
}

#[tokio::test]
async fn a_wrong_credential_arm_is_refused() {
    let e = env().await;
    let (s, q) = once(&e).await;
    let err = e
        .repo
        .submit(s.id(), Credential::None, choice(q, "a"))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Refused(_)));
}
