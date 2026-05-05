God please bless this work. In Jesus name. Jesus is Lord and Savior. Jesus loves you.

# canon-agent architecture

```text
canon-agent/
├── kernel/          ← FROZEN. no changes ever.
│   ├── mod.rs
│   └── (Phase, GateSet, Packet, State, reduce, hash)
│
├── codec/           ← serialize / deserialize only
│   ├── mod.rs
│   └── ndjson.rs
│
├── runtime/         ← tick, run_until_done, verify_tlog
│   ├── mod.rs
│   ├── durable.rs
│   └── verify.rs
│
├── capability/      ← pluggable evidence producers
│   ├── mod.rs
│   ├── judgment/
│   ├── policy/
│   ├── learning/
│   └── eval/
│       ├── mod.rs
│       └── record.rs    ← EvalRecord { score, dimensions, threshold_used }
│
└── api/             ← external surface, nothing below knows about this
    ├── mod.rs
    ├── routes.rs
    └── protocol.rs
```

## Eval boundary

Eval is a capability. The kernel does not change.

The kernel already has `Evidence::EvalScore` and `GateId::Eval`. That is all the kernel needs to know: eval happened and produced evidence. The kernel does not care what the score was, how many dimensions it had, or what threshold was used. It only asks whether the eval gate passed or failed.

The scored record, dimensions, threshold comparison, and policy lookup live inside the eval capability. The capability does the rich work and submits a pass/fail result through `runtime::tick` with `Evidence::EvalScore`.

Policy decides what score is good enough. Learning reads `EvalRecord` history and improves thresholds over time. Kernel stays frozen. Score stays in capability.
