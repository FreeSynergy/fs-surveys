//! Guarantee 6: conditions decide what is asked, and refer back only.

mod common;

use common::*;
use fs_store::ObjectId;
use fs_surveys_core::{
    Anonymity, Answer, AnswerValue as V, Condition, ConditionOp, Error, Question,
    QuestionKind as K, Survey,
};

fn ans(q: ObjectId, value: V) -> Answer {
    Answer { question: q, value }
}

async fn refused(e: &Env, s: &Survey, a: Vec<Answer>) {
    let r = e.repo.submit(s.id(), human(), a).await;
    assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
}

#[tokio::test]
async fn conditions_look_back_only_and_decide_what_is_asked() {
    let e = env().await;
    let s = e
        .repo
        .create("S", Anonymity::AnonymousOpen, &[])
        .await
        .unwrap();
    let (_, a) = e
        .repo
        .add_question(s.id(), pick().required())
        .await
        .unwrap();
    let (_, b) = e
        .repo
        .add_question(s.id(), Question::new(K::Text, "Why a?").required())
        .await
        .unwrap();
    let (_, c) = e
        .repo
        .add_question(s.id(), Question::new(K::Text, "Later"))
        .await
        .unwrap();
    let cond = |q, op, v: &str| Condition {
        question: q,
        op,
        value: v.into(),
    };
    for (target, cs) in [
        (a, vec![cond(a, ConditionOp::Answered, "")]),
        (a, vec![cond(c, ConditionOp::Answered, "")]),
        (b, vec![cond(ObjectId::new(), ConditionOp::Answered, "")]),
        (b, vec![cond(a, ConditionOp::Equals, "nope")]),
        (b, vec![cond(a, ConditionOp::AtLeast, "x")]),
    ] {
        let r = e.repo.set_conditions(s.id(), target, cs).await;
        assert!(
            matches!(r, Err(Error::Refused(_) | Error::Invalid(_))),
            "{r:?}"
        );
    }
    e.repo
        .set_conditions(s.id(), b, vec![cond(a, ConditionOp::Equals, "a")])
        .await
        .unwrap();
    let r = e.repo.move_question(s.id(), b, 0).await; // would put b before a
    assert!(matches!(r, Err(Error::Refused(_))));
    let r = e.repo.remove_question(s.id(), a).await; // b depends on a
    assert!(matches!(r, Err(Error::Refused(_))));
    let s = e.repo.publish(s.id()).await.unwrap();
    let text = |q, t: &str| ans(q, V::Text(t.into()));
    refused(&e, &s, [choice(a, "a"), vec![]].concat()).await; // asked and required
    refused(&e, &s, [choice(a, "b"), vec![text(b, "x")]].concat()).await; // not asked, answered
    e.repo
        .submit(s.id(), human(), choice(a, "b"))
        .await
        .unwrap(); // not asked, not required
    e.repo
        .submit(
            s.id(),
            human(),
            [choice(a, "a"), vec![text(b, "because")]].concat(),
        )
        .await
        .unwrap();
}
