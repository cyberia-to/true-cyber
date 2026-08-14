---
title: true-cyber
tags: cyber, cli
crystal-type: source
crystal-domain: cyber
alias: cyber cli, true-cyber
---
# true-cyber

product CLI for [cyber](https://cyber.page) — binary `cyber`, crate `true-cyber`.

```bash
cargo install true-cyber
cyber sync
```

default network: **spacepussy-test** — the soft3 chaosnet (local node at `http://127.0.0.1:7780`).

```bash
cyber sync                 # probe spacepussy-test
cyber network              # endpoints
cyber manifesto
cyber help
```

## launch

operator guide for standing up spacepussy-test: [soft3/docs/launch](https://cyber.page/soft3/docs/launch) (source: `soft3/docs/launch.md`).

## what this is not

`space-pussy` and `bostrom` on [cybernode](https://cybernode.ai) are **cosmos-sdk bootloader** chains (go-cyber). they are migration sources for the soft3 network. they are not the product default. `cyber sync -n space-pussy` is rejected on purpose.

## faces

day-one needs only `true-cyber`. soft3 / cyb remain optional deeper faces:

```bash
cyber soft3 …
cyber cy …
```

## toolchain (advanced)

```bash
cyber tools
cyber source
cyber install --all
```

## license

cyber license: don't trust. don't fear. don't beg.
