# fs-surveys – Design

`fs-surveys` is the surveys program of FreeSynergy. This repo holds the **program core** (`fs-surveys-core`, `fs-surveys-methods`),
the **Cap'n Proto root** (`fs-surveys-server`) and the **window** (`fs-surveys-app`, no drawing code). Contract: `surveys.capnp` in
`fs-interfaces` v0.0.11 (draft, provisional: no survey frontends are analysed yet). Program docs:
`fs-documentation/docs/en/programs/apps/surveys/`.

## State

Version 0.0.1 (AI participants and open surveys; round 3: type descriptions, root server, generated Admin window).

| Area | State |
|---|---|
| Surveys: create, edit, questions, conditions, state machine, classify, delete | done |
| Named answers: save, resume, submit, replace | done |
| Anonymous: separate unlogged ballot store, one box per survey, batches, tokens through `TokenDelivery` | done; tokens have no real delivery until fs-messenger plugs in |
| Results: summary, evaluate, CSV export; `filter` and rows for `named` only | done; DDI `Unsupported` |
| Ten methods, held-back bound | done |
| Widening over groups through `GroupLookup` | done, tested with real `fs-groups-core` |
| Decisions linked to task IRIs | done |
| `survey.publicLink`, `poll.embed` | `Unsupported`; `publish` refuses a `public` survey |
| AI participants: permission per survey (default no), marking by the host, doubt = AI, results apart in every reading | done, **provisional**, see "AI participants" |
| Open surveys: claim "Are you an AI?", pattern counts shown, voluntary details | done, **provisional** |
| AI participants on `public` surveys | `Unsupported` with the rest of `public` |
| AI guard against a token handed to an AI, and against a caller who names someone else | **not possible**, stated limits below |
| capnp server | root only: `supportReport` and `types` (see "Round 3"); designer, respond, results, decisions, subscribe, capabilities are **not served** (the generated method answers "unimplemented") |
| Tests | see `cargo test`, real stores |

Checked by an independent reviewer over three rounds before the tag; two rounds found serious errors, and the last found nothing serious.

## AI participants (thread 5, item 7; owner 2026-11-07; all provisional)

Rule: an AI takes part in a survey only if its creator allows it **for that survey** (default: no). Its answers are **always counted and
shown apart** from the answers of people: in every total, summary, evaluation (all ten methods), CSV export, interim result and event. Whether
a participant is an AI is told by the **marking** the host injects, never by what the participant says about itself. No MCP tool for this.

### Pattern map

| Concern | Pattern (`coding.md` §2) | Why |
|---|---|---|
| Who is an AI: `AiMarking` | Strategy (injected, like `fs_groups_core::AiMarking`) | This crate does not depend on fs-people; the host wires the answer (for example to the role book). `Some(false)` = known human; `Some(true)` = AI; `None` = no answer. |
| May this class take part: `Participation` | Specification | A pure predicate over (survey, class), testable alone; submit, save, invite and token spending all ask the same one, so no path has its own copy of the rule. |
| Counting apart: `Lanes` in reading | Template method | One flow (collect, summarise, evaluate, export) run once for people and once for AIs; the class decides which rows go in. A method cannot opt out, because the flow lives in one place and the ten strategies only see rows of one class. |
| Class of a participant: `Class` | Value object | `Human` or `Ai`, decided once at submit (or at invite for tokens) and kept with the answer. |

Names not in the table are not used. Decorator was considered for the separate evaluation and rejected: nothing wraps the same trait here;
the separation needs a second pass over the rows, which is the Template method.

### Doubt counts as AI (decision, provisional)
A participant is a **human only when the marking answers `Some(false)` for an identified person**. No marking configured, no answer (`None`),
no identity (`Credential::None`), `Some(true)`: all are **AI**. An error of the marking is passed on and never counts as human. Reason: the
two possible mistakes are not equal. Counting an AI as a person falsifies the result everybody reads; refusing a person who the host could
not vouch for costs one retry. This follows `fs-groups` (`AiMarking`: no answer is treated like "agent").

### Anonymous surveys
* `anonymousOnce`: the class is fixed **when the token is issued**, from the marking of the invited person, and kept as a lane of the token
  hash in the box (not linked to the person). Spending a token puts the ballot into the lane of the token. An AI that is not allowed gets no
  token (`InviteReport.refused`). **Limit, stated:** a person can hand their token to an AI. The core cannot see that; the ballot counts as the
  class of the person who was invited. An invited AI can likewise hand its token to a person. The same holds for any poll with tokens.
* `anonymousOpen` (**owner 2026-11-07, replaces the rule "no person from the host = AI"**): anyone may answer, with no proof and no
  account. The respondent hands in `Credential::Open(OpenMeta)`: the answer to **"Are you an AI?"** (required; without it there is no
  ballot), the time the form was open, and the value of a hidden field. The answer is a **claim, not a proof**, and the interface says so.
  A declared AI goes to the AI lane and needs `aiAllowed`; anyone else goes to the people lane. `Credential::None` is refused.
  A host that vouches for a person still passes `Credential::Person`; then the marking decides and the person is not kept.
* **Patterns of open answers** (counted, shown, never used to drop an answer): sent faster than `1500 ms + 600 ms` per answer; hidden field
  filled; the same choice in every one of at least three choice or scale answers; a burst (more than ten open submits in one minute, counted
  in memory only); no timing given; the same free text (twelve characters or more) in several ballots, counted when read. They are **counts
  per lane**, kept with the pending ballots and released with their batch, never one flag per ballot, so no flag can be tied to an answer
  and no arrival time is written. `Summary.patterns` shows them beside the figures; `ballots` always holds every released answer.
  Limit: patterns are hints. A careful bot passes all of them, and a fast person trips some.
* **Voluntary details after the answers** (`add_voluntary`): `key = value` pairs (age group, role), at most five per call, key up to 40 and
  value up to 100 characters (longer ones are dropped, so a name cannot be written into the result), at most 10 000 sets per survey. They
  are kept in a list of the box with no link to a ballot, inserted at a random position, and shown in `Summary.voluntary` and the CSV
  **only after the survey closed** and when at least `minGroup` sets were given; while it is open the list would change with every
  arrival. Stated limit: on an open survey anyone can add sets, so the count is of sets, not of people.
  The patterns count only open claims that were looked at (`signals.ballots`); a ballot of a host-vouched person or from an older box adds
  none, so `signals.ballots` can be smaller than `ballots`. The burst counter is a hint: on a busy survey with more than ten answers a minute
  every further answer counts as part of a burst.
* `public`: unchanged, `Unsupported` (`survey.publicLink`). When it is built it needs the same decision.
* Batches: each class has its own pending list, batches and `minGroup` rule, so AI ballots never pad a batch of people (nor the other way round).
* **The core does not authenticate.** `person` and `by` come from the host, as everywhere in this crate. If the host lets a caller name any
  person, the marking of that person is used. A caller who can name a human can answer as a human.

### Permission
`aiAllowed` and `creator` are fields of the survey. Only `creator` can change `aiAllowed` (`Designer.setAiAllowed(id, by, allowed)`), and only
while the survey is a draft or open, and **granting needs a creator who is a human by the marking** (an AI cannot allow AIs). A survey made
without `createBy` has no creator, so AI participation can never be allowed for it. Withdrawing the permission leaves every earlier AI answer
where it is, apart; it blocks new AI answers, saves, invitations and the spending of tokens issued to AIs (a refused spend keeps the token).

### Smaller decisions and limits
* The class is fixed when the answer is submitted (or the token issued) and kept with it. If the marking changes later, old answers stay where they are.
* A stored named response without the `fs:ai` flag counts as an AI answer (doubt). A survey without `fs:aiAllowed` counts as "no".
* No MCP tool exists for any of this (owner, 2026-11-07): not for the permission, not for the AI part of the results.
* Whether `Credential::person` is really the caller is the host's job (capnp server, thread 3). The core cannot check it.
* An AI that is invited and allowed can hand its token to a person; the ballot counts as AI. Same limit, other direction.

### Results
All ten methods run through `Lanes::run` in `repo/reading.rs`, which is also what summary and filter use (both exports take one lane directly with `Lanes::of`, in `repo/export.rs`); there is no
reading of answers outside it. `Summary` and `Evaluation` carry an `ai` part (same shape, never nested) next to the part for people; it is present when the survey allows AI
or holds AI answers. `ballots`, counts, `heldBack` and winners at the top level are people only. `Results.exportAi` is a second CSV with a first
column `participants` = `ai`; `export` is people only. A named response says whether it is an AI answer. An AI part is shown when the survey allows AIs or holds AI answers; otherwise `ai` is absent. `SurveyChange::AiAnswered` is a
separate event; `Answered` is people only.

## Decisions (the owner may change them; all provisional)

### One survey is one object
A survey is one `StoredObject` of envelope type `OrderedCollection`, `context=surveys`, IRI `https://<instance>/surveys/<id>`:
AS2 JSON with `orderedItems` = AS2 `Question`s (`oneOf`/`anyOf`/`closed`) plus the profile in `fs:` members (namespace
`https://freesynergy.net/ns#`). So every edit is atomic. Questions change only in `draft`.

### Anonymity and the store
`fs-objects` logs every change with its exact time and full before/after images, sets `updated` itself, and hands the log to the hub;
ids are UUIDv7, which contain the time. An anonymous answer stored as an ordinary object would carry all of that. So:

* **Anonymous answers live in a separate store, the ballot store.** The host passes a second store, as `Arc<dyn UnloggedObjects>` (fs-objects 0.3), which is never connected to the hub. Nothing anonymous goes into the main store.
* **One box per survey.** (It has a lane for people and one for AIs, see "AI participants"; what follows is true of each lane.) The ballot store holds one object per survey with: `pending` ballots, released `batches` (`{stage, ballots}`) and unspent token hashes. A ballot has no person, time or id. Each submit inserts at a random position.
* **No log, no time.** Boxes are written with `put_unlogged`: no activity exists at any moment, so a crash leaves nothing. Both envelope times are the Unix epoch. The box id contains only when the box was created.
* **Results come in batches.** Reads only see released batches. Release happens when the survey closes or widens: if `pending` has at least `minGroup` ballots they become one shuffled batch of the current stage. Smaller rests stay and are reported as `heldBack`. Reading never releases, so comparing two readings, or closing and reopening, shows at least a batch. `minGroup` defaults to 5 and is at least 3 on anonymous surveys.
* **Per question too.** A batch counts for a question only if it has at least `minGroup` answers to that question, or a conditional question would expose one answer through the difference between readings.
* **Answers a batch rule suppresses count as held back for that question,** or a result could look decided when the suppressed answers would flip it.
* **Empty answers are no answers:** empty text, lists and score sets are dropped at submit and in evaluation, and do not count toward a batch.
* **While a survey is open** `heldBack` in a summary reads 0 and `evaluate` gives no winner, because either would change with each submit and show arrival times. `evaluate` still reports what the per-question rule suppressed, which only changes at a release. After close, `heldBack` is real.
* **`subscribe` never sees ballots**, because they are in the other store. State changes and named answers only.
* **Tokens (`anonymousOnce`):** 256 random bits from `getrandom`, sent straight to the person through the `TokenDelivery` port; the caller never sees them. Only `sha256(token)` is kept, in the box, sorted, not linked to a person. The main store keeps the invited people apart. Delivery happens outside the survey lock, so a slow messenger does not stop answers; a person being invited is marked in memory, so two parallel invites send one token. The second caller is told `already`, even if that delivery then fails; inviting again sends a token. A token used in the instant before it is recorded is refused, and works on retry. Submitting removes the hash in the same write as the ballot. Spent and unknown tokens get the same error.
* **Held-back answers:** `evaluate_with_held_back` gives no winner when they could change the outcome. There is a provable bound for plurality, approval, majority and supermajority; all other methods stay undecided as long as any answer is held back. `heldBack` must be shown.
* **Anonymous results show:** choice counts, scale counts, means; number and date only as a mean; free text shuffled. No rows and no `filter`.

**Stated limits, to be shown by the interface:** whoever runs the machine can read memory while a ballot or a token passes, can copy the file between two writes and see the difference, and old pages may remain until the database reuses them. Unanimous groups stay recognisable. Two conditional questions asked of nearly the same people can expose someone through the difference between their results; the batch rule cannot prevent that, so such a design needs care. An organiser who closes and reopens after each answer can watch `heldBack` rise; each reopening is on record in the activity log. On `anonymousOpen` the organiser can add own ballots and thereby isolate someone else's in a batch; only `anonymousOnce` limits who can answer, and `public` needs the protection against mass answers. Named answers and the list of invited people stay in the main store's activity log after a delete, until its retention removes them. Answers can stay held back, up to `minGroup - 1` per survey plus those the per-question rule suppresses in each batch, so more in total; a vote can stay undecided because of them, and the evaluation says how many. Provisional; the owner decides. `anonymousOnce` is therefore not called fully anonymous.

### Locks
One lock per survey, shared by clones of the repo, from read to save. Two repos or two processes on one store are **not** safe: there is one `SurveyRepo` per store. A compare-and-swap in `fs-objects` would lift that.

### Named answers
One `StoredObject` of store type `SurveyResponse` per person and survey in the main store, `inReplyTo` the survey, index fields `survey`, `person`, `submitted`. `save` keeps a draft; submit sets `submitted`; submitting again replaces while the survey is open.

### Anonymity is fixed at publishing
It can be edited in `draft` only. Widening has no such parameter, so it cannot get weaker.

### Widening
`widen(id, parent)`: survey must be `open` and bound to a group; `parent` must be an ancestor of a group in `scope`, asked through the `GroupLookup` port. Releases pending per the batch rule, then `stage += 1`, `scope` gains `parent`. The survey stays one object. New people are invited with `invite`. Tests use real `fs-groups-core` behind an adapter in the test directory (dev-dependency only).

### Ports (Strategy, ADR 0043)
`GroupLookup` (`ancestors(group)`, `members(group)` as person IRIs) and `TokenDelivery` (`send(person, survey, token)`). No delivery configured: `invite` on `anonymousOnce` answers `Unsupported`. fs-messenger and the capnp clients plug in later.

### Decisions
Store type `Decision` (AS2 has none), main store, `context=surveys`, content in `fs:` members; index `survey`, `task` (multi-valued). A decision names survey, question, method, threshold, stage, outcome, note and task IRIs. The task itself stays in fs-tasks; `record` does not require the method's own winner, because people decide. It is refused if a choice id does not exist.

### Methods are a separate crate
`fs-surveys-methods` is pure: ballots in, `Tally` out, one strategy per method, a registry from `Method` to strategy. The same ballots can be evaluated by every method whose ballot form fits; a form that does not fit is an error, not a guess.
Not in the contract yet, so not built: quadratic voting, delegation, clustering, matrix questions.

### Not built in this version
Each answers `Unsupported` where the contract has a method, and is listed in `capabilities`: `survey.publicLink` (next step in this repo: adaptive proof of work bound to the survey, global limit, quality score, "all" beside "checked"). DDI export. No contract method yet, nothing here: `poll.date`, `poll.federate`, `template.use`, `survey.translate`.

**Data from 0.1.0 (provisional).** Ballots and token hashes stored without a class count as AI, like named answers without `fs:ai`: in doubt never among people. No server existed, so no real data is affected (`tests/ai_legacy.rs`).


## Round 3: type descriptions, root server, Admin window (ADR 0066, 0065)

### What a generated frontend is told
Four data types, each `DescribesType` in the core (`crates/fs-surveys-core/src/describe/`): `Survey` (AS2 `OrderedCollection` + `fs:` profile),
`Question` (AS2 `Question` + profile), `SurveyResponse` (the contract's `NamedResponse`; AS2 has no answer) and `Decision` (own profile; AS2 has none).
Anonymous ballots are not a data type: they have no id and no time on purpose, they only exist as the `submit` command.
Commands come from **one table**, `crates/fs-surveys-core/src/commands/` (one file per type): name, kind, requirement ids, `supported`, reason, input, result.
Every command id of `requirements.md` is in it (test `every_requirement_command_is_in_the_table`, which reads a snapshot of the file in
`tests/data/requirements.md`; set `FS_SURVEYS_REQUIREMENTS` to the file in fs-documentation to compare with the current one) and every command is in the
description sent over the wire, the unsupported ones with `supported = false` (`fs-types` v0.0.11). The flag is copied in one place (`describe/build.rs`, `carry_flag`).
The backend refuses an unsupported command with a typed `RunError::Unsupported` (`Gated::try_run`), and reports it locked through `Backend::supported`.

| Type | Commands carried | Commands described, not carried (`supported = false`) |
|---|---|---|
| `Survey` | create, edit (the `change` as an `Edit`), anonymity, classify, publish, close, reopen, archive, delete, invite (also `survey.tokens`), widen, aiAllowed, find, summary, evaluate, filter, export, exportAi | subscribe (the core has a subscription, a window has no listener to keep), embed (`poll.embed`), publicLink, date (`poll.date`), template (`template.use`), translate, federate, thread, groups (`result.groups`), delegate (`vote.delegate`), compose, modes |
| `Question` | add, edit (`Edit`), remove, move, setConditions (`logic.set`), find, types | none |
| `SurveyResponse` | open, save, find (resume), submit, voluntary | challenge (`survey.publicLink`) |
| `Decision` | record, find, linkTask, unlinkTask | none |

`question.validate` is not a command of its own: required questions and bounds are checked in `save` and `submit` (the requirement id is carried by both).
`survey.tokens` is carried by `invite` (tokens are delivered through `TokenDelivery`; none is configured in the window, so the core reports it unsupported there).
The `supported` flag follows the core where it can: a test checks that every command marked supported is reported supported by the core's own capability report.

### Pattern map (`coding.md` section 2)

| Where | Pattern | Why |
|---|---|---|
| `commands/` | Registry of rows | the one source for description, backend rule and the requirements test |
| `describe/` | Registry entry (trait first) + Builder | `DescribesType` per type; description built from the table |
| `fs-surveys-server` `SurveysServer` | Adapter | generated `surveys::Server` over the core's `Surveys` trait; `types` hands out `fs_types::describe::CatalogServer` |
| app `Gated`/`Typed` | Proxy (protection) | one place refuses unsupported commands; a backend cannot forget it |
| app `survey.rs`, `question.rs`, `response.rs`, `decision.rs` | Repository over the core | `Backend` of the frontend |
| app `ctx.rs` | Adapter (sync to async) | the frontend calls synchronously with its session locked; the core is async |
| app `answers.rs` | Adapter + Strategy by question kind | answers as text lines, there is no basic type for an answer |
| app `charts.rs` | Adapter | summary to a Vega-Lite spec for `fs-components-charts`; no drawing |
| app `operator.rs` | Strategy (injected `AiMarking`) | the operator is a person, everybody else counts as an AI (doubt rule) |

### Decisions of this round (provisional, reported)
* **Types and keys.** Choice keys are kebab case (`anonymous-open`), as the description rule demands; the core's camel-case names are converted in `describe::keys`.
  There is no number or yes/no basic type yet: numbers are text fields (`minGroup`, `min`, `thresholdPercent` ...) that the backend parses (a bad one is refused), yes/no is a choice `yes`/`no`.
* **Change** follows the rule of 2026-11-10 (`Edit`, field by field, no JSON): a field not in the edit stays, an empty one is cleared. `Survey.edit` takes `name`, `anonymity`, `minGroup`, `categories`;
  `Question.edit` takes every field of the question. Other commands that look like changes (publish, close, anonymity, setConditions, linkTask ...) are kind `other` on a record, because their input is not an `Edit`.
* **Ids.** A record id is an IRI: `https://<instance>/surveys/<id>`, a question `<survey>/questions/<id>`, a decision `https://<instance>/decisions/<id>`. Parsing takes the last segment.
* **Lists.** `find` is the standard `find` (term, list of references). `SurveyResponse.find` needs a survey **and** a person (the core resumes one person's answer); without both the list is empty.
* **Answers** are text lines `<question IRI>=<value>`; several values joined by `,`; scores `<choice>:<score>`.
* **Who is a person.** The window marks the operator (the actor) as a human, everybody else gets no answer and so counts as an AI (the core's doubt rule). A named answer typed for another person is therefore refused unless the survey allows AIs.
* **Refusals.** `BackendError` of the frontend has no variant for "a rule said no" or "not supported". Mapped: unsupported and unknown command to `NotFound`, a rule of the core to `Rejected` on field `id` with `NotInList`, an unreadable value to `Rejected` with `Character`, a store failure to `Unavailable`. The typed reasons stay available through `Gated::try_run` (`RunError`). Reported to thread A.
* **Versions.** `fs-interfaces` and `fs-types` v0.0.11 everywhere. `fs-admin-frontend` at fs-components rev `7b47a65` (the commit that takes v0.0.11: `Backend::change`, `Backend::supported`, the record id beside the input; the pinned `4688cee` does not compile against v0.0.11). `fs-components-charts` at main rev `5c6ed69`. Blinc through `blinc-app-fix` `5180d4e` and `blinc-layout-fix` `cb17db8`, the pins of `fs-tasks`.

### Unsupported (named, not faked)
* **Charts in the window.** `question_chart` makes a validated chart (`fs-components-charts`) from a summary, tested, but the generated Admin frontend has no place to show a chart yet: a result is shown as lines of text (`counts`). A chart hook in `fs-admin-frontend` (a result field of a chart kind) is thread A's; until then no chart is drawn in this window.
* **Server.** `designer`, `respond`, `results`, `decisions`, `subscribe`, `capabilities` of `surveys.capnp` are not served; the window talks to the core in its own process. No `serve` command.
* **Render code.** The repo has none: there was no `gui.rs` or `surface.rs` before this round, so nothing had to move to `fs-components`.
* **Question types** matrix and ranking-by-matrix (`question.types`, optional): not in the core (`QuestionKind` has eight kinds, the `types` command lists them).
* **DDI export**, **anonymous results rows**, **public link**: as in the table above.
* Needs `Respond.voluntary`, `survey.tokens` delivery (messenger), group lookup (`fs-groups`): not wired in the window; the core reports them unsupported there.
* No Apple, Windows or simple frontend (R6, later).
