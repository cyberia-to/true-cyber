---
title: true-cyber
tags: cyber, cli
crystal-type: source
crystal-domain: cyber
alias: cyber cli, true-cyber
---
# true-cyber

the product CLI for [cyber](https://cyber.page) — binary name `cyber`, crate name `true-cyber` (crates.io `cyber` is taken).

```bash
cargo install true-cyber
cyber sync
```

default network is **space-pussy** — the chaosnet. override with `-n bostrom`.

```bash
cyber sync                 # probe space-pussy RPC
cyber sync -n bostrom      # probe bostrom
cyber network              # endpoints for the default network
cyber network bostrom
cyber manifesto
cyber help
```

## faces

day-one needs only `true-cyber`. soft3 and cyb are optional deeper faces:

```bash
cyber soft3 …              # forward if `soft3` is on PATH
cyber cy …                 # forward if `cy` is on PATH
```

network presets and probe come from the [soft3](https://crates.io/crates/soft3) crate in-process — no separate install for `cyber sync`.

## toolchain (advanced)

build the full stack from source under `$CYBER_ROOT` (default `~/cyber`):

```bash
cyber tools
cyber source               # clone missing repos
cyber install --all        # build tools onto ~/.cargo/bin
cyber install zheng        # one tool
cyber zheng …              # run a registered tool
```

## license

cyber license: don't trust. don't fear. don't beg.
