# fs-surveys

FreeSynergy program for surveys, polls and group decisions: anonymity where it is needed, several decision methods over the same ballots, and a survey that can widen from a small group to its parent groups.

* `crates/fs-surveys-methods`: the decision methods, pure code.
* `crates/fs-surveys-core`: the program core (no GUI, no capnp server yet). See `DESIGN.md`.
* The contract is `surveys.capnp` in [fs-interfaces](https://github.com/FreeSynergy/fs-interfaces) (tag v0.46.0).

Licence: MIT OR Apache-2.0.
