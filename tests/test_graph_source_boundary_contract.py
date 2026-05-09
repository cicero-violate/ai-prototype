#!/usr/bin/env python3
from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "03-graph-source-of-truth.md"


class GraphSourceBoundaryContractTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.doc = DOC.read_text(encoding="utf-8")
        cls.compact_doc = " ".join(cls.doc.split())

    def test_subproject_boundary_contract_names_all_authorities(self) -> None:
        for token in (
            "## Subproject Boundary Contract",
            "ai/ root runtime",
            "canon-rustc-v3/",
            "graph-editor/",
            "owns: graph schema constants",
            "owns: compiler-wrapper capture",
            "owns: human-facing graph inspection/editing workflows",
        ):
            self.assertIn(token, self.doc)

    def test_runtime_boundary_excludes_wrapper_and_editor_ownership(self) -> None:
        for token in (
            "must not own: compiler-wrapper capture internals or interactive graph editing UI",
            "TLog admission rules",
            "validation/report evidence",
        ):
            self.assertIn(token, self.compact_doc)

    def test_wrapper_boundary_excludes_runtime_policy_and_ui(self) -> None:
        for token in (
            "must not own: runtime state-machine transitions",
            "TLog admission",
            "policy promotion",
            "mutation/query UI behavior",
        ):
            self.assertIn(token, self.doc)

    def test_editor_boundary_excludes_capture_runtime_and_policy(self) -> None:
        for token in (
            "may read: graph.json, typed graph operation contracts, mutation receipts",
            "must not own: compiler-wrapper capture",
            "runtime state-machine transitions",
            "policy promotion",
        ):
            self.assertIn(token, self.doc)

    def test_boundary_exceptions_require_documentation_and_contract_tests(self) -> None:
        for token in (
            "No subproject may silently replace another subproject's authority",
            "boundary exception",
            "covered by a contract test",
        ):
            self.assertIn(token, self.compact_doc)


if __name__ == "__main__":
    unittest.main()
