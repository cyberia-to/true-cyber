---
title: true-cyber
tags: cyber, cli
crystal-type: source
crystal-domain: cyber
alias: cyber cli, cyber bootstrap
---
# true-cyber

the cyber stack, from source to PATH

```
cargo install true-cyber
cyber install --all
```

`true-cyber` installs one binary — `cyber` — the bootstrap face of the
[cyber](https://cyber.page) stack. it carries the toolset registry and turns a
bare machine into a working cyber system built from source:

```
cyber                  status — repos and tools
cyber sync             clone the source repos into $CYBER_ROOT (default ~/cyber)
cyber install --all    build every tool and link it onto PATH
cyber install zheng    build one tool
cyber zheng …          run a tool by name
cyber graph            serve the knowledge graph locally
```

## the toolset

| tool | role |
|------|------|
| hemera | particle identity |
| mudra | neuron identity |
| nox | virtual machine |
| rune | dynamic language |
| trident | provable language |
| neural | graph language |
| strata | field algebras |
| glia | model runtime |
| radio | transport |
| tape | wire framing |
| foculus | consensus |
| cybergraph | cyberlink processor |
| bbg | authenticated state |
| inf | query engine |
| lens | polynomial commitment |
| zheng | proof system |
| eidos | formal verification |
| tru | truth layer |
| prysm | visual protocol |
| lytics | visitor analytics |
| cyb | graphical interface |
| cy | terminal face |

tools build from the [cyberia-to](https://github.com/cyberia-to) repos. public
repos clone anonymously; a few are still private and are skipped until they
open — the rest of the stack works without them.

## requirements

git, cargo (stable). builds run with `RUSTC_BOOTSTRAP=1` — some workspaces use
nightly features on the stable compiler.

## license

cyber license: don't trust. don't fear. don't beg.
