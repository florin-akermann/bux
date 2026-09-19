# CodeScene MCP — approved tools

Reference for the CodeScene MCP allow/deny list.
The rule lives in `AGENTS.md`; this file is the enumeration.

## Use only these tools

- `explain_code_health` — learn about the Code Health metric
- `explain_code_health_productivity` — business case for Code Health
- `code_health_review` — detailed review of a single file
- `code_health_score` — quick numeric score for a file
- `pre_commit_code_health_safeguard` — check staged changes before commit
- `analyze_change_set` — branch-level review before PR

## Never use these

- `get_config` / `set_config` — no need to inspect or change server config
- `code_health_refactoring_business_case` — no ROI/business-case analysis needed
- Any socio-technical feature (knowledge silos, code ownership, team coupling)
