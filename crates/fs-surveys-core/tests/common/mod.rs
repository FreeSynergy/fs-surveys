//! Shared helpers of the integration tests: real in-memory stores, real groups, a collecting delivery.

#![allow(dead_code)] // each test file uses a different subset

use std::sync::{Arc, Mutex};

pub mod ai;

use async_trait::async_trait;
use fs_store::{ActivityFilter, ObjectId, Query, Store, TursoStore};
use fs_surveys_core::{
    Anonymity, Answer, AnswerValue, Credential, Error, GroupLookup, Question, QuestionKind, Result,
    Survey, SurveyRepo, TokenDelivery,
};

pub const ME: &str = "https://example.org/users/owner";

/// Delivery that remembers every token it was given.
#[derive(Default)]
pub struct Collect(pub Mutex<Vec<(String, String)>>);

impl Collect {
    pub fn token_of(&self, person: &str) -> String {
        let all = self.0.lock().unwrap();
        all.iter().find(|(p, _)| p == person).unwrap().1.clone()
    }
    pub fn all(&self) -> Vec<String> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|(_, t)| t.clone())
            .collect()
    }
}

#[async_trait]
impl TokenDelivery for Collect {
    async fn send(&self, person: &str, _survey: ObjectId, token: &str) -> Result<()> {
        self.0.lock().unwrap().push((person.into(), token.into()));
        Ok(())
    }
}

/// Delivery that always fails.
pub struct Failing;

#[async_trait]
impl TokenDelivery for Failing {
    async fn send(&self, _p: &str, _s: ObjectId, _t: &str) -> Result<()> {
        Err(Error::Invalid("mail server is down".into()))
    }
}

/// `GroupLookup` over the real `fs-groups-core` (built on fs-store 0.2, so ids pass as text).
pub struct Groups(pub fs_groups_core::GroupRepo);

fn to_old(id: ObjectId) -> fs_store_02::ObjectId {
    id.to_string().parse().unwrap()
}
fn from_old(id: fs_store_02::ObjectId) -> ObjectId {
    id.to_string().parse().unwrap()
}

#[async_trait]
impl GroupLookup for Groups {
    async fn ancestors(&self, group: ObjectId) -> Result<Vec<ObjectId>> {
        let found = self.0.ancestors(to_old(group)).await.unwrap();
        Ok(found.into_iter().map(from_old).collect())
    }
    async fn members(&self, group: ObjectId) -> Result<Vec<String>> {
        let found = self.0.effective(to_old(group)).await.unwrap();
        Ok(found.into_iter().map(|m| m.person).collect())
    }
}

/// Everything a test needs.
pub struct Env {
    pub repo: SurveyRepo,
    pub main: Arc<TursoStore>,
    pub ballots: Arc<TursoStore>,
    pub delivery: Arc<Collect>,
    pub groups: fs_groups_core::GroupRepo,
}

pub async fn env() -> Env {
    let main = Arc::new(TursoStore::open_in_memory().await.unwrap());
    let ballots = Arc::new(TursoStore::open_in_memory().await.unwrap());
    let old: Arc<dyn fs_store_02::Store> =
        Arc::new(fs_store_02::TursoStore::open_in_memory().await.unwrap());
    let groups = fs_groups_core::GroupRepo::new(old, ME).with_instance("example.org");
    let delivery = Arc::new(Collect::default());
    let repo = SurveyRepo::new(main.clone(), ballots.clone(), ME)
        .with_instance("example.org")
        .with_groups(Arc::new(Groups(groups.clone())))
        .with_delivery(delivery.clone())
        .with_ai_marking(Arc::new(ai::Marks));
    Env {
        repo,
        main,
        ballots,
        delivery,
        groups,
    }
}

/// The credential of a person on an `anonymousOpen` survey: only used to ask the marking.
pub fn human() -> Credential {
    Credential::Person(person("h"))
}

pub fn person(n: &str) -> String {
    format!("https://example.org/people/{n}")
}

/// A single-choice question with choices `a`, `b`, `c`.
pub fn pick() -> Question {
    Question::new(QuestionKind::SingleChoice, "Which?").with_choices(&[
        ("a", "A"),
        ("b", "B"),
        ("c", "C"),
    ])
}

/// A published survey with one single-choice question; returns the survey and the question id.
pub async fn open_survey(e: &Env, anonymity: Anonymity) -> (Survey, fs_store::ObjectId) {
    let s = e.repo.create("Lunch", anonymity, &[]).await.unwrap();
    let (_, q) = e
        .repo
        .add_question(s.id(), pick().required())
        .await
        .unwrap();
    (e.repo.publish(s.id()).await.unwrap(), q)
}

pub fn choice(q: ObjectId, c: &str) -> Vec<Answer> {
    vec![Answer {
        question: q,
        value: AnswerValue::Choice(c.into()),
    }]
}

/// `n` anonymous submits of `c`.
pub async fn submit_open(e: &Env, s: &Survey, q: ObjectId, c: &str, n: usize) {
    for _ in 0..n {
        e.repo.submit(s.id(), human(), choice(q, c)).await.unwrap();
    }
}

/// Everything a store holds, as one text: objects and the activity log.
pub async fn raw(store: &TursoStore) -> String {
    let objects = store.query(&Query::new()).await.unwrap();
    let log = store.activities(&ActivityFilter::default()).await.unwrap();
    format!(
        "{}\n{}",
        serde_json::to_string(&objects).unwrap(),
        serde_json::to_string(&log).unwrap()
    )
}

pub async fn log_len(store: &TursoStore) -> usize {
    store
        .activities(&ActivityFilter::default())
        .await
        .unwrap()
        .len()
}

/// The raw box content of a survey.
pub async fn box_json(e: &Env, s: &Survey) -> serde_json::Value {
    use fs_store::traits::UnloggedObjects;
    e.ballots
        .read_unlogged(s.id())
        .await
        .unwrap()
        .unwrap()
        .content
}

pub fn count(sum: &fs_surveys_core::Summary, q: ObjectId, choice: &str) -> u32 {
    let t = sum.questions.iter().find(|t| t.question == q).unwrap();
    t.counts
        .iter()
        .find(|c| c.choice == choice)
        .map_or(0, |c| c.count)
}

impl Env {
    /// True if the box holds no token hash (or there is no box yet).
    pub async fn ballots_empty_tokens(&self, s: &Survey) -> bool {
        use fs_store::traits::UnloggedObjects;
        match self.ballots.read_unlogged(s.id()).await.unwrap() {
            None => true,
            Some(o) => o.content["tokens"].as_array().unwrap().is_empty(),
        }
    }
}
