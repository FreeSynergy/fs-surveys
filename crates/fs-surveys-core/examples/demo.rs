//! A survey in one go: design, invite with tokens, answer anonymously, release, evaluate, decide.
//! The host tells who is an AI (`AiMarking`); an AI is not invited unless the creator allowed it.
//!
//! Run with `cargo run -p fs-surveys-core --example demo`.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use fs_objects::{ObjectId, TursoStore};
use fs_surveys_core::{
    ALL_STAGES, AiMarking, Anonymity, Answer, AnswerValue, Credential, Designer, Error, Method,
    Question, QuestionKind, SurveyRepo, TokenDelivery,
};

/// The host's role book in miniature: `bot-*` are AIs, other people are humans.
struct RoleBook;

#[async_trait]
impl AiMarking for RoleBook {
    async fn is_ai(&self, person: &str) -> Result<Option<bool>, Error> {
        Ok(Some(person.contains("/bot-")))
    }
}

/// Stands in for a messenger: keeps the tokens so the demo can answer with them.
#[derive(Default)]
struct Outbox(Mutex<Vec<String>>);

#[async_trait]
impl TokenDelivery for Outbox {
    async fn send(&self, _person: &str, _survey: ObjectId, token: &str) -> Result<(), Error> {
        self.0.lock().unwrap().push(token.to_string());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (main, ballots) = (
        Arc::new(TursoStore::open_in_memory().await?),
        Arc::new(TursoStore::open_in_memory().await?),
    );
    let outbox = Arc::new(Outbox::default());
    let repo = SurveyRepo::new(main, ballots, "https://example.org/users/owner")
        .with_instance("example.org")
        .with_delivery(outbox.clone())
        .with_ai_marking(Arc::new(RoleBook));

    let survey = repo
        .create_by(
            "Where do we meet?",
            Anonymity::AnonymousOnce,
            &[],
            "https://example.org/people/ada",
        )
        .await?;
    repo.edit(
        survey.id(),
        "Where do we meet?",
        Anonymity::AnonymousOnce,
        3,
    )
    .await?;
    let place = Question::new(QuestionKind::SingleChoice, "Place")
        .with_choices(&[("hall", "Town hall"), ("park", "Park")])
        .required();
    let (_, question) = repo.add_question(survey.id(), place).await?;
    repo.publish(survey.id()).await?;

    let people: Vec<String> = ["ada", "bob", "cy", "di", "bot-1"]
        .iter()
        .map(|n| format!("https://example.org/people/{n}"))
        .collect();
    let report = repo.invite(survey.id(), &people, &[]).await?;
    println!(
        "invited {} people, refused {} AI; tokens went to the outbox, not to us",
        report.invited.len(),
        report.refused.len()
    );

    let tokens = outbox.0.lock().unwrap().clone();
    for (token, pick) in tokens.into_iter().zip(["hall", "hall", "park", "hall"]) {
        let answer = vec![Answer {
            question,
            value: AnswerValue::Choice(pick.into()),
        }];
        repo.submit(survey.id(), Credential::Token(token), answer)
            .await?;
    }
    let before = repo.summary(survey.id(), ALL_STAGES).await?;
    println!(
        "before close: {} ballots shown, {} held back",
        before.ballots, before.held_back
    );
    repo.close(survey.id()).await?;
    let after = repo.summary(survey.id(), ALL_STAGES).await?;
    println!("after close: {} ballots shown", after.ballots);
    for count in &after.questions[0].counts {
        println!("  {} = {}", count.choice, count.count);
    }
    for method in [
        Method::Plurality,
        Method::Majority,
        Method::Supermajority {
            threshold_percent: 80,
        },
    ] {
        let evaluation = repo
            .evaluate(survey.id(), question, method, ALL_STAGES)
            .await?;
        println!(
            "{method:?}: winners {:?} {}",
            evaluation.winners, evaluation.note
        );
    }
    let decision = repo
        .record(
            survey.id(),
            question,
            Method::Majority,
            ALL_STAGES,
            vec!["hall".into()],
            "Town hall it is",
        )
        .await?;
    println!("decided: {:?}", decision.outcome);
    Ok(())
}
