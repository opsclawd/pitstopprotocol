# Pivot Plan (USDC spec-first + TDD)

1. Freeze old implementation line in archive tag/branch.
2. Merge spec pack (this PR) with no feature code.
3. PR-1: unit tests only (canonical IDs + math + timestamp rules).
4. PR-2+: instruction-by-instruction TDD in lifecycle order.
5. Enforce PR protocol: file-by-file summary + independent review + fix pass + final review-summary comment.
