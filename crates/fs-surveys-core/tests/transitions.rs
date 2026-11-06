//! Guarantee 6: state transitions and publish rules; each refusal.

mod common;

use common::*;
use fs_surveys_core::{Anonymity, Credential, Error, Question, QuestionKind as K, SurveyState};

#[tokio::test]
async fn transitions_and_publish_rules() {
    let e = env().await;
    let s = e.repo.create("S", Anonymity::Named, &[]).await.unwrap();
    assert!(
        matches!(e.repo.publish(s.id()).await, Err(Error::Refused(_))),
        "no question"
    );
    let one = Question::new(K::SingleChoice, "One").with_choices(&[("a", "A")]);
    let (_, q1) = e.repo.add_question(s.id(), one).await.unwrap();
    assert!(
        matches!(e.repo.publish(s.id()).await, Err(Error::Refused(_))),
        "one choice"
    );
    let dup = Question::new(K::SingleChoice, "Dup").with_choices(&[("a", "A"), ("a", "B")]);
    assert!(
        matches!(
            e.repo.add_question(s.id(), dup).await,
            Err(Error::Invalid(_))
        ),
        "same id twice"
    );
    e.repo
        .edit_question(s.id(), pick().with_id(q1))
        .await
        .unwrap();
    for step in 0..3 {
        let r = match step {
            0 => e.repo.close(s.id()).await,
            1 => e.repo.reopen(s.id()).await,
            _ => e.repo.archive(s.id()).await,
        };
        assert!(
            matches!(r, Err(Error::InvalidTransition { .. })),
            "draft: {r:?}"
        );
    }
    assert!(
        matches!(
            e.repo
                .submit(s.id(), Credential::Person(person("a")), choice(q1, "a"))
                .await,
            Err(Error::Refused(_))
        ),
        "draft takes no answers"
    );
    e.repo.publish(s.id()).await.unwrap();
    assert!(matches!(
        e.repo.publish(s.id()).await,
        Err(Error::InvalidTransition { .. })
    ));
    assert!(matches!(
        e.repo.archive(s.id()).await,
        Err(Error::InvalidTransition { .. })
    ));
    assert!(matches!(
        e.repo.reopen(s.id()).await,
        Err(Error::InvalidTransition { .. })
    ));
    assert!(
        matches!(
            e.repo.add_question(s.id(), pick()).await,
            Err(Error::Refused(_))
        ),
        "questions change in draft only"
    );
    let closed = e.repo.close(s.id()).await.unwrap();
    assert_eq!(closed.as2()["closed"], true);
    assert!(matches!(
        e.repo
            .submit(s.id(), Credential::Person(person("a")), choice(q1, "a"))
            .await,
        Err(Error::Refused(_))
    ));
    assert!(matches!(
        e.repo.close(s.id()).await,
        Err(Error::InvalidTransition { .. })
    ));
    assert!(matches!(
        e.repo.publish(s.id()).await,
        Err(Error::InvalidTransition { .. })
    ));
    assert_eq!(
        e.repo.reopen(s.id()).await.unwrap().as2().get("closed"),
        None
    );
    e.repo.close(s.id()).await.unwrap();
    assert_eq!(
        e.repo.archive(s.id()).await.unwrap().state,
        SurveyState::Archived
    );
    assert!(matches!(
        e.repo.reopen(s.id()).await,
        Err(Error::InvalidTransition { .. })
    ));
}
