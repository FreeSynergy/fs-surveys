# FreeSynergy Surveys — ask, and decide together

**A poll where an anonymous answer really is anonymous, not just labelled that way.**

FreeSynergy Surveys lets a person or a group ask questions, collect answers and read the result. Answers can carry a name, or be anonymous. A survey can start in a small group and later widen to the larger group around it, keeping the answers already given.

What is different here: anonymous answers do not go into the ordinary store at all. They go into a separate box that keeps no log and no time, with answers shuffled. Results appear only in batches of at least a minimum group size, so reading twice cannot reveal one person's answer. If answers are held back and could change the outcome, the program says it cannot name a winner yet. A decision can be linked to the tasks that follow from it.

## What it can do today

Latest tag: `v0.0.1`.

- Create and edit surveys with questions and conditions, through the stages draft, open, closed.
- Named answers: save a draft, resume, submit, replace while open.
- Anonymous answers in a separate box, with one-time tokens for "one answer per invited person".
- Results: summary, evaluation with counting methods such as plurality, approval, majority and supermajority, and a CSV export.
- Widen a survey over groups, tested with the real Groups core.
- Link a decision to task identifiers.
- Tests: 107, with real stores. Checked by an independent reviewer over three rounds; two rounds found serious errors and the last found none.

## What is not there yet

- No window and no server.
- Tokens have no real delivery until the messenger plugs in.
- Public links and embedding are not possible; DDI export is not supported.
- Anonymity has stated limits: someone running the machine can still read memory while a ballot passes. The design file lists them, and a window must show them.
- One survey store per process.
- The interface is a draft, marked provisional, because no survey frontends have been analysed yet.

## Part of FreeSynergy

Surveys belongs to the democracy and project-management suites. FreeSynergy is a family of programs that are free for everyone, forever, and that keep your data yours. The documentation, the manifesto and the overall plan live in [fs-documentation](https://github.com/FreeSynergy/fs-documentation); the reading version is at <https://freesynergy.github.io/fs-documentation/>. Start with the [manifesto](https://freesynergy.github.io/fs-documentation/manifesto/) if you want to know why we do this.


## For developers

- Workspace crates: `fs-surveys-core` (core, no GUI) and `fs-surveys-methods` (counting methods).
- The contract is `surveys.capnp` in [fs-interfaces](https://github.com/FreeSynergy/fs-interfaces) (draft, provisional). Surveys owns it.
- Build and test: `cargo build --workspace` and `cargo test --workspace`.
- Design and decisions: [DESIGN.md](DESIGN.md).
- Licence: MIT OR Apache-2.0.
