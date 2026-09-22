---
tags: true-cyber, audit, build
date: 2026-09-22
---
# true-cyber does not build from a clean checkout

`main` at `b58589f` ("wip: rewrite cyber as authenticated native graph
client on cyb-core GraphSession", committed 2026-09-16) references a
`cyb-core` API — `GraphSession`, `native_mirror::Mirror`, and the whole
`robot` module (`Registry`, `custody`, `host::{Host, lookup, nonce}`,
`remote::{self, Transport, hex, parse_id}`) — that does not exist in
`cyb-core` 0.1.0 at `../cyb/core`, and never has: `cyb-core/src/lib.rs`
exports only `cell`, `chroma`, `intent`, `money`, `sense`, `signal`, plus
re-exports from `cybergraph`/`foculus`. Neither `GraphSession` nor
`native_mirror` appears anywhere in the `cyb` repository's history; the
`robot` the commit expects is a different thing from cyb's `robot` *world*
(`shell/src/worlds/robot/`), an unrelated naming collision. `true-cyber` is
one of the phase-1 binaries (component table, dependency closure of
`soft3` node, `cyb`, `cy`, `true-cyber`, `foculus`, `neuron`, `tok`, `tru`)
and one of the six release-train builds (calendar week 5, lane B); it has
been broken on `main` for six days without anyone noticing because nothing
runs its build.

Verified 2026-09-22, `cargo 1.95.0` (`rustc 1.95.0`), macOS arm64, commit
`b58589f130f4f80dd8b2752f98704842372aa2d9`:

```
$ cargo check --tests
error[E0433]: cannot find `robot` in `cyb_core`
 --> src/http.rs:2:15
error[E0432]: unresolved imports `cyb_core::GraphSession`, `cyb_core::native_mirror`, `cyb_core::robot`
 --> src/main.rs:5:5
error[E0432]: unresolved imports `cyb_core::GraphSession`, `cyb_core::robot`
 --> tests/native_client.rs:2:5
error[E0433]: cannot find `robot` in `cyb_core`
   --> tests/native_client.rs:218:19
error: could not compile `true-cyber` (bin "cyber" test) due to 3 previous errors
error: could not compile `true-cyber` (test "native_client") due to 3 previous errors
```

Six compile errors across three files (`src/main.rs`, `src/http.rs`,
`tests/native_client.rs`), all the same root cause: the `robot`/
`GraphSession`/`native_mirror` surface the rewrite depends on was never
added to `cyb-core`. This is not a stale-pin gap like rows 38/44/46/47/48
(no version bump closes it) — it is an in-progress rewrite committed
directly to `main` with its dependency half missing. `git log --oneline -5`
on `main`:

```
b58589f wip: rewrite cyber as authenticated native graph client on cyb-core GraphSession
6cd1d8e feat!: 0.7 cyber is a cell — pull frames, replay, verify the root yourself
7f89e86 feat!: 0.6 parse cybermark status natively; serde_json is gone
5c30c9e fix: 0.5.1 show soft3 engine fields; clarify soft3 owns the network
8996a8c fix: 0.5 thin install — drop soft3 stack dep
```

`6cd1d8e` (the last commit before the WIP rewrite) is the last known-green
point for this binary; reverting `b58589f` there is the fastest path back
to a shippable `true-cyber` if the rewrite does not land before the
2026-10-23 feature freeze.

The fix is not a slice this pass can size: it requires either designing
and implementing `cyb-core::{GraphSession, native_mirror, robot}` (a real
new API surface, owned by the `cyb` repo, shape unknown from the calling
side alone) or reverting the WIP commit and re-doing the rewrite once that
API exists. Both are follow-up work, not audit scope.
