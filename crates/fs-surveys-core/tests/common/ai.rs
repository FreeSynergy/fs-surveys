//! Helpers of the AI participant tests: a marking, surveys that allow or forbid AIs, a fixture with
//! one question for each ballot form, and answers that differ between people and AIs.

#![allow(dead_code)]

use async_trait::async_trait;
use fs_objects::ObjectId;
use fs_surveys_core::Designer as _;
use fs_surveys_core::{
    AiMarking, Anonymity, Answer, AnswerValue as V, Credential, Error, Method, OptionScore,
    Question, QuestionKind as K, Result, Survey,
};

use super::{Env, choice, person, pick};

/// The host's marking: `bot-*` are AIs, other people are humans, anything else is unknown.
pub struct Marks;

#[async_trait]
impl AiMarking for Marks {
    async fn is_ai(&self, who: &str) -> Result<Option<bool>> {
        Ok(match who.strip_prefix("https://example.org/people/") {
            Some(n) if n.starts_with("bot-") => Some(true),
            Some(_) => Some(false),
            None => None,
        })
    }
}

/// A marking that fails.
pub struct Broken;

#[async_trait]
impl AiMarking for Broken {
    async fn is_ai(&self, _who: &str) -> Result<Option<bool>> {
        Err(Error::Invalid("role book is down".into()))
    }
}

pub fn bot(n: &str) -> String {
    person(&format!("bot-{n}"))
}

/// The creator of the surveys below.
pub fn owner() -> String {
    person("owner")
}

/// A published survey made by `owner()`, with one required question; `allow` sets `aiAllowed`.
/// Anonymous surveys get `minGroup` 3.
pub async fn survey(e: &Env, anonymity: Anonymity, allow: bool) -> (Survey, ObjectId) {
    let s = e
        .repo
        .create_by("Lunch", anonymity, &[], &owner())
        .await
        .unwrap();
    if anonymity != Anonymity::Named {
        e.repo.edit(s.id(), "Lunch", anonymity, 3).await.unwrap();
    }
    let (_, q) = e
        .repo
        .add_question(s.id(), pick().required())
        .await
        .unwrap();
    if allow {
        e.repo.set_ai_allowed(s.id(), &owner(), true).await.unwrap();
    }
    (e.repo.publish(s.id()).await.unwrap(), q)
}

/// Submit as a named participant.
pub async fn say(e: &Env, s: &Survey, who: &str, answers: Vec<Answer>) -> Result<()> {
    e.repo
        .submit(s.id(), Credential::Person(who.into()), answers)
        .await
}

/// `n` named answers of `c` by people called `{prefix}-0..`; bots when `ai`.
#[allow(clippy::many_single_char_names)] // short fixture names (env, survey, question)
pub async fn many(e: &Env, s: &Survey, q: ObjectId, c: &str, n: usize, ai: bool) {
    for i in 0..n {
        let who = if ai {
            bot(&i.to_string())
        } else {
            person(&format!("h{i}"))
        };
        say(e, s, &who, choice(q, c)).await.unwrap();
    }
}

/// The five questions of `fixture`: single, multiple, ranking, scores 0..10, scores 0..1.
pub const FORMS: usize = 5;

/// All ten methods and the question (index into `fixture`) whose ballots suit them.
pub const TEN: [(Method, usize); 10] = [
    (Method::Plurality, 0),
    (Method::Majority, 0),
    (
        Method::Supermajority {
            threshold_percent: 60,
        },
        0,
    ),
    (Method::Approval, 1),
    (Method::Schulze, 2),
    (Method::InstantRunoff, 2),
    (Method::Borda, 2),
    (Method::Score, 3),
    (Method::SystemicConsensing, 3),
    (Method::Consent, 4),
];

/// A named survey with one question for each ballot form, published; `allow` lets AIs in.
pub async fn fixture(e: &Env, allow: bool) -> (Survey, [ObjectId; FORMS]) {
    let abc = [("a", "A"), ("b", "B"), ("c", "C")];
    let mut ten = Question::new(K::OptionScores, "ten").with_choices(&abc);
    (ten.min, ten.max) = (0, 10);
    let mut one = Question::new(K::OptionScores, "one").with_choices(&abc);
    (one.min, one.max) = (0, 1);
    let questions = [
        Question::new(K::SingleChoice, "single").with_choices(&abc),
        Question::new(K::MultipleChoice, "multiple").with_choices(&abc),
        Question::new(K::Ranking, "ranking").with_choices(&abc),
        ten,
        one,
    ];
    let s = e
        .repo
        .create_by("Forms", Anonymity::Named, &[], &owner())
        .await
        .unwrap();
    let mut ids = vec![];
    for q in questions {
        ids.push(e.repo.add_question(s.id(), q).await.unwrap().1);
    }
    if allow {
        e.repo.set_ai_allowed(s.id(), &owner(), true).await.unwrap();
    }
    let ids = <[ObjectId; FORMS]>::try_from(ids).unwrap();
    (e.repo.publish(s.id()).await.unwrap(), ids)
}

fn strs(l: &[&str]) -> Vec<String> {
    l.iter().map(|x| (*x).into()).collect()
}

fn points(l: [i32; 3]) -> V {
    V::Scores(
        ["a", "b", "c"]
            .iter()
            .zip(l)
            .map(|(c, score)| OptionScore {
                choice: (*c).into(),
                score,
            })
            .collect(),
    )
}

/// What people answer (`ai` false) and what AIs answer: the opposite, so any mixing shows.
pub fn answers(q: &[ObjectId; FORMS], ai: bool) -> Vec<Answer> {
    let values = if ai {
        [
            V::Choice("c".into()),
            V::Choices(strs(&["b", "c"])),
            V::Ranking(strs(&["c", "b", "a"])),
            points([0, 1, 9]),
            points([0, 1, 1]),
        ]
    } else {
        [
            V::Choice("a".into()),
            V::Choices(strs(&["a"])),
            V::Ranking(strs(&["a", "b", "c"])),
            points([9, 1, 0]),
            points([1, 1, 1]),
        ]
    };
    q.iter()
        .zip(values)
        .map(|(question, value)| Answer {
            question: *question,
            value,
        })
        .collect()
}

/// Five people and four bots answer the fixture.
pub async fn fill(e: &Env, s: &Survey, q: &[ObjectId; FORMS], people: usize, bots: usize) {
    for i in 0..people {
        say(e, s, &person(&format!("h{i}")), answers(q, false))
            .await
            .unwrap();
    }
    for i in 0..bots {
        say(e, s, &bot(&i.to_string()), answers(q, true))
            .await
            .unwrap();
    }
}
