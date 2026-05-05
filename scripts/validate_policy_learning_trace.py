#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


TRACE_FN = "learning_policy_llm_feedback_loop_drives_judgment"

TRACE_TOKENS = (
    "run_until_done(State::ready(), RuntimeConfig::default())",
    "PolicyPromotion::from_tlog(&learned_tlog, 1)",
    "PolicyStore::default()",
    "policy.promote_feedback(promotion)",
    "ContextRecord::from_packet_memory",
    "LlmStructuredAdapter::record_from_context(&context, &policy, 11)",
    "llm.prompt.policy_hash",
    "policy.fingerprint()",
    "llm.prompt.policy_version",
    "policy.latest_version()",
    "crate::api::routes::handle_command",
    "Command::SubmitEvidence(llm.submission())",
    "response.event.to, Phase::Plan",
    "response.event.evidence, Evidence::JudgmentRecord",
    "state.gates.judgment.status, GateStatus::Pass",
    "verify_tlog(&tlog)",
)

PROMOTION_TOKENS = (
    "pub struct PolicyPromotion",
    "pub fn from_tlog",
    "Evidence::PolicyPromotion",
    "promoted_policy_hash",
    "source_seq",
)

STORE_TOKENS = (
    "pub fn try_append",
    "pub fn append_durable",
    "key_to_id(entry.key)?",
    "InvalidPromotion",
    "PolicyEntry",
    "pub fn fingerprint",
    "pub fn latest_version",
)

LEARNING_TO_POLICY_TOKENS = (
    "impl PolicyStore",
    "pub fn promote_feedback",
    "PolicyStoreError::InvalidPromotion",
    "PolicyEntry",
    "self.try_append(PolicyEntry",
    "key: POLICY_FEEDBACK_HASH",
    "pub fn promote_feedback_durable",
    "self.append_durable(",
)

POLICY_LAYER_FORBIDDEN_TOKENS = (
    "pub fn promote_feedback",
    "pub fn promote_feedback_durable",
    "PolicyPromotion",
)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return ""


def function_body(text: str, name: str) -> str:
    marker = f"fn {name}"
    start = text.find(marker)
    if start < 0:
        return ""
    brace = text.find("{", start)
    if brace < 0:
        return ""
    depth = 0
    for index in range(brace, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[brace + 1:index]
    return ""


def token_check(name: str, text: str, tokens: tuple[str, ...]) -> dict[str, Any]:
    missing = [token for token in tokens if token not in text]
    return {
        "name": name,
        "status": "pass" if not missing else "fail",
        "checked": len(tokens),
        "missing": missing,
    }


def forbidden_token_check(name: str, text: str, tokens: tuple[str, ...]) -> dict[str, Any]:
    present = [token for token in tokens if token in text]
    return {
        "name": name,
        "status": "pass" if not present else "fail",
        "checked": len(tokens),
        "missing": [f"forbidden:{token}" for token in present],
    }


def validate(root: Path) -> dict[str, Any]:
    lib = read(root / "src" / "lib.rs")
    promote = read(root / "src" / "capability" / "learning" / "promote.rs")
    store = read(root / "src" / "capability" / "policy" / "store.rs")
    trace_body = function_body(lib, TRACE_FN)
    checks = [
        {
            "name": "trace_function_present",
            "status": "pass" if trace_body else "fail",
            "checked": 1,
            "missing": [] if trace_body else [TRACE_FN],
        },
        token_check("trace_function_contract", trace_body, TRACE_TOKENS),
        token_check("policy_promotion_contract", promote, PROMOTION_TOKENS),
        token_check("learning_to_policy_append_contract", promote, LEARNING_TO_POLICY_TOKENS),
        token_check("policy_store_append_only_contract", store, STORE_TOKENS),
        forbidden_token_check("policy_store_no_learning_promotion_contract", store, POLICY_LAYER_FORBIDDEN_TOKENS),
    ]
    missing = [item for check in checks for item in check["missing"]]
    return {
        "schema_version": 1,
        "validation": "policy_learning_trace",
        "status": "pass" if not missing else "fail",
        "trace_function": TRACE_FN,
        "checks": checks,
        "missing": missing,
        "missing_count": len(missing),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate source-level learning→policy→LLM→judgment trace evidence.")
    parser.add_argument("--root", default=".")
    parser.add_argument("--report")
    args = parser.parse_args()
    result = validate(Path(args.root))
    text = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.report:
        Path(args.report).parent.mkdir(parents=True, exist_ok=True)
        Path(args.report).write_text(text, encoding="utf-8")
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return 0 if result["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())