---
review_agents: [code-simplicity-reviewer, security-sentinel, performance-oracle, architecture-strategist]
plan_review_agents: [code-simplicity-reviewer]
---

# Review Context

This repository contains example applications and planning documents for
Bankai- and SP1-based workflows.

Focus review attention on:
- correctness of example application foundations
- security and operational risks in on-chain or proof-related code paths
- simplicity over abstraction in Rust code
- drift between documentation and implementation

Do not flag `docs/plans/*.md` or `docs/solutions/*.md` for cleanup just because
they are generated pipeline artifacts.
