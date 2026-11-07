//! What this provider reports it can do.
//!
//! Pattern: Null Object (report) — lists every capability id named in `surveys.capnp`; what is not
//! built, or not configured, says so with the reason (ADR 0045).

use fs_interfaces::Capabilities;

use super::handle::SurveyRepo;

const SUPPORTED: [&str; 23] = [
    "survey.create",
    "survey.aiAllowed",
    "survey.edit",
    "survey.anonymity",
    "survey.classify",
    "question.add",
    "question.edit",
    "question.move",
    "question.validate",
    "logic.set",
    "survey.publish",
    "survey.close",
    "survey.invite",
    "survey.open",
    "response.save",
    "response.submit",
    "result.summary",
    "result.export",
    "result.filter",
    "survey.method",
    "decision.record",
    "decision.read",
    "decision.link",
];

impl SurveyRepo {
    pub(super) fn support(&self) -> Capabilities {
        let mut c = Capabilities::new("fs-surveys-core");
        for id in SUPPORTED {
            c.supported(id);
        }
        c.supported("survey.subscribe");
        if self.delivery.is_some() {
            c.supported("survey.tokens");
        } else {
            c.unsupported(
                "survey.tokens",
                "no token delivery is configured; it is possible",
            );
        }
        if self.groups.is_some() {
            c.supported("survey.widen");
        } else {
            c.unsupported(
                "survey.widen",
                "no group lookup is configured; it is possible",
            );
        }
        c.unsupported(
            "survey.publicLink",
            "not built yet: adaptive proof of work, global limit, quality score",
        );
        c.unsupported(
            "poll.embed",
            "the host program embeds a survey; the core has no call for it",
        );
        c
    }
}
