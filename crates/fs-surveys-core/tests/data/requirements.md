# Surveys — Requirements

What `FreeSynergy.Surveys` (`fs-surveys`) must and can do. The program owns AS2 `Question` and the decision topic
([interface catalogue](../../../interfaces/index.md)). Anything another program owns is reached through that program's
interface and is not specified again here (ADR 0050). Background: [research note](https://github.com/FreeSynergy/fs-documentation/blob/main/temp/research/surveys-landscape.md).

**Provisional (2026-11-06).** No frontends have been analysed for this category, so nothing here has passed the usual
rule (ADR 0006, 0013). Everything comes from our architecture (*arch.*) or is general knowledge of such programs (⚠️),
and must be checked against analysed programs before the contract is written. The research note gives a landscape by
purpose, but it is not a frontend analysis.

Status: `implemented` · `planned` · `proposed`

## Required

| Capability | Basis | Data / standard | Status |
|------------|-------|-----------------|--------|
| Create and edit a survey (`survey.create`, `survey.edit`) | ⚠️ | AS2 `Collection` + own profile; the envelope belongs to `fs-store` | proposed |
| Add, edit, reorder questions (`question.add`, `question.edit`, `question.move`) | ⚠️ | AS2 `Question`; gap beyond it | proposed |
| Question types: single, multiple, text, number, date, scale (`question.types`) | ⚠️ | `oneOf`, `anyOf`; rest gap | proposed |
| Required questions and validation (`question.validate`) | ⚠️ | gap | proposed |
| Publish, close, reopen (`survey.publish`, `survey.close`) | ⚠️ | AS2 `closed` | proposed |
| Choose the anonymity level and show it to respondents (`survey.anonymity`) | *arch.* ([design](design.md#anonymity)) | gap | proposed |
| Invite people and groups (`survey.invite`) | ⚠️ | people via the `fs-people` interface, groups via `fs-groups`, the message via `fs-messenger` | proposed |
| Open, answer, save and resume, submit (`survey.open`, `response.save`, `response.submit`) | ⚠️ | gap | proposed |
| See results as totals and charts (`result.summary`) | ⚠️ | derived; the charts come from the [charts component](../../../components/charts.md), shown as a result dashboard view | proposed |
| Export results (`result.export`) | ⚠️ | CSV ⚠️; DDI ⚠️ | proposed |
| Mark sensitive surveys and answers (`survey.classify`) | *arch.* (ADR 0029) | DPV | proposed |
| Embed a poll in another program (`poll.embed`) | *arch.* (ADR 0037) | AS2 `Question`; shown as a compact workbench, the host program only embeds it | proposed |
| Notify about changes (`subscribe`) | *arch.* (ADR 0021) | AS2 activities | proposed |
| Choose the decision method per question: majority, approval, score, ranked, consent and others (`survey.method`) | *arch.* (ADR 0049, point 8; [design](design.md#decision-types)) | gap; one strategy per method | proposed |
| Widen a survey from a small group to its parent groups, results carried over (`survey.widen`) | *arch.* (ADR 0049, point 8; [design](design.md#widening-to-parent-groups)) | groups from the `fs-groups` interface; the survey stays one object | proposed |

## Decisions as objects (provisional, from contract draft)

`decision.record`, `decision.read` and `decision.link` keep a decision as an object linked to tasks. `survey.subscribe` follows
what is happening, as in the other contracts.

## Optional

| Capability | Basis | Data / standard | Status |
|------------|-------|-----------------|--------|
| Skip logic (`logic.set`) | ⚠️ | gap | proposed |
| Matrix and ranking questions (`question.types`) | ⚠️ | gap | proposed |
| One answer per person with anonymity (`survey.tokens`) | *arch.* | gap | proposed |
| Open survey, no login, with protection against mass answers (`survey.publicLink`) | *arch.* (ADR 0049, point 8; [design](design.md#open-surveys-without-login)) | gap | proposed |
| Date poll, result goes to the calendar (`poll.date`) | ⚠️ | the event is created through the `fs-calendar` interface, which owns `Event`; no event model here | proposed |
| Templates and a question bank (`template.use`) | ⚠️ | wiki templates (ADR 0031), kept by `fs-wiki` | proposed |
| Filter and cross-tabulate results (`result.filter`) | ⚠️ | derived | proposed |
| Translated surveys (`survey.translate`) | ⚠️ | AS2 `nameMap` | proposed |
| Federated poll to other servers (`poll.federate`) | *arch.* | AS2 `Question`, ActivityPub | proposed |
| Decide inside the discussion thread, outcome kept with the reasoning (`survey.thread`) | Loomio | gap | proposed |
| Show clusters of opinion and common ground instead of a single winner (`result.groups`) | Polis | derived; maths component | proposed |
| Delegate one's vote per topic, revocable (`vote.delegate`) | LiquidFeedback | gap | proposed |
| Participation spaces with chosen components (`space.compose`) | Decidim | gap | proposed |
| Several participation modes on one site: debate, proposal, budget, poll (`space.modes`) | Consul Democracy, Decidim | gap | proposed |

## Not planned (deliberately)

- Binding elections: they need much stronger guarantees and belong to the [democracy suite](../../suites/democracy/index.md).
- Tracking respondents (advertising, device fingerprints): contrary to the purpose of the program.
- Automatic analysis of answers by external services: answers must not leave the program.
