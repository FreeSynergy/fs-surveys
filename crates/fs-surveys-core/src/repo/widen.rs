//! Widening a survey to a parent group.
//!
//! Pattern: Strategy (port) — `GroupLookup` answers which groups are ancestors. Anonymity has no
//! parameter here, so it cannot get weaker; earlier results stay in their stage.

use fs_interfaces::Unsupported;
use fs_store::ObjectId;

use super::handle::SurveyRepo;
use crate::error::{Error, Result};
use crate::model::{Survey, SurveyState};

impl SurveyRepo {
    /// `survey.widen`: `parent` must be an ancestor of a group in `scope`.
    pub async fn widen(&self, id: ObjectId, parent: ObjectId) -> Result<Survey> {
        let _guard = self.lock_for(id).await;
        let mut s = self.require(id).await?;
        let lookup = self
            .groups
            .as_ref()
            .ok_or_else(|| Unsupported::new("survey.widen", "no group lookup is configured"))?;
        if s.state != SurveyState::Open {
            return Err(Error::Refused(format!(
                "the survey is {:?}, not open",
                s.state
            )));
        }
        if s.scope.is_empty() {
            return Err(Error::Refused("the survey is not bound to a group".into()));
        }
        if s.scope.contains(&parent) {
            return Err(Error::Refused("that group is already asked".into()));
        }
        let mut is_ancestor = false;
        for g in &s.scope {
            is_ancestor |= lookup.ancestors(*g).await?.contains(&parent);
        }
        if !is_ancestor {
            return Err(Error::Refused(
                "that group is not a parent of a group asked".into(),
            ));
        }
        let stage = s
            .stage
            .checked_add(1)
            .filter(|n| *n != crate::model::ALL_STAGES);
        let stage = stage.ok_or_else(|| Error::Refused("too many stages".into()))?;
        self.release_box(&s).await?;
        s.stage = stage;
        s.scope.push(parent);
        self.save(&s).await
    }
}
