//! Domain architecture sketch.
//!
//! This module is intentionally not wired into `lib.rs`.
//! It sketches the domain layer that should sit above raw world inputs and
//! below existing capability evidence production.
//!
//! Boundary:
//! - domain: finance/global-intelligence/business/trading interpretation
//! - capability: evidence records, receipts, tools, verification
//! - kernel: frozen state machine
//! - runtime: deterministic transitions and replay
//!
//! Non-goals while unwired:
//! - no direct `State` mutation
//! - no direct `TLog` append
//! - no changes to `kernel`
//! - no changes to `runtime::reduce`
//! - no live trading path
//!
//! Conceptual lifecycle:
//! 1. DomainSignal
//! 2. DomainContext
//! 3. DomainJudgment
//! 4. DomainPlan
//! 5. DomainRiskEnvelope
//! 6. DomainEval
//! 7. DomainPromotionCandidate
//!
//! TODO:
//! - define stable domain records before adding behavior
//! - define schema versions before serialization
//! - define deterministic hash identity before receipts
//! - avoid mutating kernel state directly
//! - route future domain output through capability evidence submissions
//! - require provenance, confidence, uncertainty, and risk envelope fields
//! - keep trading sandbox-only until risk and verification contracts exist
//!
//! Proposed future Rust modules, not declared yet:
//! - contracts
//! - scoring
//! - global_intelligence
//! - finance
//! - business
//! - trading
