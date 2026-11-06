//! Decisions: what people decided, linked to tasks.
//!
//! Pattern: Repository — a `Decision` object in the main store, index fields `survey` and `task`
//! (multi-valued). The task itself stays in fs-tasks; only its IRI is kept here.

use fs_store::{ObjectId, Order, Query, StoredObject};
use fs_surveys_methods::Method;
use serde_json::json;

use super::consts::{CONTEXT, DECISION_TYPE};
use super::envelope::context;
use super::handle::SurveyRepo;
use crate::error::{Error, Result};
use crate::model::{AS2_TYPE, Cursor, Decision, Page, method_parse, method_text};

fn read(o: &StoredObject) -> Option<Decision> {
    let c = &o.content;
    let text = |k: &str| c.get(k)?.as_str().map(String::from);
    let strings = |k: &str| serde_json::from_value::<Vec<String>>(c.get(k)?.clone()).ok();
    Some(Decision {
        envelope: o.envelope.clone(),
        survey: text("fs:survey")?.parse().ok()?,
        question: text("fs:question")?.parse().ok()?,
        method: method_parse(&text("fs:method")?)?,
        outcome: strings("fs:outcome")?,
        note: text("fs:note")?,
        tasks: strings("fs:tasks")?,
        stage: u16::try_from(c.get("fs:stage")?.as_u64()?).ok()?,
    })
}

fn object(d: &Decision, ctx: &serde_json::Value) -> StoredObject {
    let content = json!({
        "@context": ctx, "type": DECISION_TYPE,
        "fs:survey": d.survey, "fs:question": d.question, "fs:method": method_text(d.method),
        "fs:outcome": d.outcome, "fs:note": d.note, "fs:tasks": d.tasks, "fs:stage": d.stage,
    });
    let mut o = StoredObject::new(d.envelope.clone(), AS2_TYPE, content)
        .with_index("survey", d.survey.to_string());
    for t in &d.tasks {
        o = o.with_index("task", t);
    }
    o
}

impl SurveyRepo {
    /// `decision.record`: refused if the question or a choice id does not exist.
    pub async fn record(
        &self,
        survey: ObjectId,
        question: ObjectId,
        method: Method,
        stage: u16,
        outcome: Vec<String>,
        note: &str,
    ) -> Result<Decision> {
        let _guard = self.lock_for(survey).await;
        let s = self.require(survey).await?;
        let q = s.question(question).ok_or(Error::NotFound {
            what: "question",
            id: question,
        })?;
        if let Some(bad) = outcome
            .iter()
            .find(|o| !q.choices.iter().any(|c| &c.id == *o))
        {
            return Err(Error::Invalid(format!(
                "{bad:?} is not a choice of {:?}",
                q.text
            )));
        }
        let mut envelope = self.fresh_envelope(DECISION_TYPE, &format!("Decision: {}", q.text));
        envelope.in_reply_to = Some(self.survey_ref(survey));
        let d = Decision {
            envelope,
            survey,
            question,
            method,
            outcome,
            note: note.into(),
            tasks: vec![],
            stage,
        };
        self.objects
            .put(object(&d, &context()), &self.actor)
            .await?;
        self.require_decision(d.id()).await
    }

    /// `decision.read`.
    pub async fn get_decision(&self, id: ObjectId) -> Result<Option<Decision>> {
        let o = self.objects.get(id).await?;
        Ok(o.filter(|o| o.envelope.object_type == DECISION_TYPE)
            .as_ref()
            .and_then(read))
    }

    async fn require_decision(&self, id: ObjectId) -> Result<Decision> {
        self.get_decision(id).await?.ok_or(Error::NotFound {
            what: "decision",
            id,
        })
    }

    /// `decision.read`: the decisions of a survey, newest first, one page.
    pub async fn list_decisions(
        &self,
        survey: ObjectId,
        cursor: Option<&Cursor>,
    ) -> Result<Page<Decision>> {
        let offset = cursor.map_or(0, Cursor::offset);
        let q = Query::new()
            .of_type(DECISION_TYPE)
            .in_context(CONTEXT)
            .with_index("survey", survey.to_string())
            .ordered(Order::PublishedDesc)
            .offset(offset)
            .limit(self.page_size + 1);
        let mut items: Vec<Decision> = self
            .reads
            .query(&q)
            .await?
            .iter()
            .filter_map(read)
            .collect();
        let next = (items.len() > self.page_size).then(|| Cursor::at(offset + self.page_size));
        items.truncate(self.page_size);
        Ok(Page { items, next })
    }

    /// `decision.read`: the decisions linked to a task.
    pub async fn of_task(&self, task: &str) -> Result<Vec<Decision>> {
        let q = Query::new().of_type(DECISION_TYPE).with_index("task", task);
        Ok(self
            .reads
            .query(&q)
            .await?
            .iter()
            .filter_map(read)
            .collect())
    }

    async fn retask(&self, id: ObjectId, f: impl FnOnce(&mut Vec<String>)) -> Result<Decision> {
        // The survey's lock, so a delete cannot orphan the decision; one lock at a time only.
        let survey = self.require_decision(id).await?.survey;
        let _guard = self.lock_for(survey).await;
        self.require(survey).await?;
        let mut d = self.require_decision(id).await?;
        f(&mut d.tasks);
        self.objects
            .put(object(&d, &context()), &self.actor)
            .await?;
        self.require_decision(id).await
    }

    /// `decision.link`: link a task IRI (twice is the same as once).
    pub async fn link_task(&self, id: ObjectId, task: &str) -> Result<Decision> {
        if task.is_empty() {
            return Err(Error::Invalid("a task IRI is empty".into()));
        }
        self.retask(id, |t| {
            if !t.iter().any(|x| x == task) {
                t.push(task.into());
            }
        })
        .await
    }

    /// `decision.link`: remove a task link.
    pub async fn unlink_task(&self, id: ObjectId, task: &str) -> Result<Decision> {
        self.retask(id, |t| t.retain(|x| x != task)).await
    }
}
