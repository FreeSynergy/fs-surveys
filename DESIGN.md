# fs-surveys – Design

`fs-surveys` is the surveys program of FreeSynergy. This repo holds the **program core**, no GUI. Contract: `surveys.capnp` in
`fs-interfaces` v0.21.0 (draft, provisional: no survey frontends are analysed yet). Program docs:
`fs-documentation/docs/en/programs/apps/surveys/`.

## State

Version 0.1.0, being built. The State table is filled in as parts are finished.

## Decisions (the owner may change them; all provisional)

### One survey is one object
A survey is one `StoredObject` of envelope type `OrderedCollection`, `context=surveys`, IRI `https://<instance>/surveys/<id>`:
AS2 JSON with `orderedItems` = AS2 `Question`s (`oneOf`/`anyOf`/`closed`) plus the profile in `fs:` members (namespace
`https://freesynergy.net/ns#`). So every edit is atomic. Questions change only in `draft`.

### Anonymity and the store
`fs-store` logs every change with its exact time and full before/after images, sets `updated` itself, and hands the log to the hub;
ids are UUIDv7, which contain the time. An anonymous answer stored as an ordinary object would carry all of that. So:

* **Anonymous answers live in a separate store, the ballot store.** The host passes a second store, as `Arc<dyn UnloggedObjects>` (fs-store 0.3), which is never connected to the hub. Nothing anonymous goes into the main store.
* **One box per survey.** The ballot store holds one object per survey with: `pending` ballots, released `batches` (`{stage, ballots}`) and unspent token hashes. A ballot has no person, time or id. Each submit inserts at a random position.
* **No log, no time.** Boxes are written with `put_unlogged`: no activity exists at any moment, so a crash leaves nothing. Both envelope times are the Unix epoch. The box id contains only when the box was created.
* **Results come in batches.** Reads only see released batches. Release happens when the survey closes or widens: if `pending` has at least `minGroup` ballots they become one shuffled batch of the current stage. Smaller rests stay and are reported as `heldBack`. Reading never releases, so comparing two readings, or closing and reopening, shows at least a batch. `minGroup` defaults to 5 and is at least 3 on anonymous surveys.
* **Per question too.** A batch counts for a question only if it has at least `minGroup` answers to that question, or a conditional question would expose one answer through the difference between readings.
* **Answers a batch rule suppresses count as held back for that question,** or a result could look decided when the suppressed answers would flip it.
* **Empty answers are no answers:** empty text, lists and score sets are dropped at submit and in evaluation, and do not count toward a batch.
* **While a survey is open** an anonymous `heldBack` reads 0 and `evaluate` gives no winner, because either would change with each submit and show arrival times. After close, `heldBack` is real.
* **`subscribe` never sees ballots**, because they are in the other store. State changes and named answers only.
* **Tokens (`anonymousOnce`):** 256 random bits from `getrandom`, sent straight to the person through the `TokenDelivery` port; the caller never sees them. Only `sha256(token)` is kept, in the box, sorted, not linked to a person. The main store keeps the invited people apart. Delivery happens outside the survey lock, so a slow messenger does not stop answers; a person being invited is marked in memory, so two parallel invites send one token. Submitting removes the hash in the same write as the ballot. Spent and unknown tokens get the same error.
* **Held-back answers:** `evaluate_with_held_back` gives no winner when they could change the outcome. There is a provable bound for plurality, approval, majority and supermajority; all other methods stay undecided as long as any answer is held back. `heldBack` must be shown.
* **Anonymous results show:** choice counts, scale counts, means; number and date only as a mean; free text shuffled. No rows and no `filter`.

**Stated limits, to be shown by the interface:** whoever runs the machine can read memory while a ballot or a token passes, can copy the file between two writes and see the difference, and old pages may remain until the database reuses them. Unanimous groups stay recognisable. Two conditional questions asked of nearly the same people can expose someone through the difference between their results; the batch rule cannot prevent that, so such a design needs care. An organiser who closes and reopens after each answer can watch `heldBack` rise; each reopening is on record in the activity log. On `anonymousOpen` the organiser can add own ballots and thereby isolate someone else's in a batch; only `anonymousOnce` limits who can answer, and `public` needs the protection against mass answers. Named answers and the list of invited people stay in the main store's activity log after a delete, until its retention removes them. Up to `minGroup - 1` answers can stay held back, so a vote can stay undecided because of them; the evaluation says how many. Provisional; the owner decides. `anonymousOnce` is therefore not called fully anonymous.

### Locks
One lock per survey, shared by clones of the repo, from read to save. Two repos or two processes on one store are **not** safe: there is one `SurveyRepo` per store. A compare-and-swap in `fs-store` would lift that.

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
