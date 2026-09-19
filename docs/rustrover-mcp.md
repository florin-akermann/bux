# RustRover MCP — where it beats alternatives

Reference for the RustRover MCP tools worth preferring over textual tooling.
The rule lives in `AGENTS.md`; this file is the enumeration.

- **Rename symbols** (`rename_refactoring`) — semantically aware; use instead of sed/find-replace.
- **Find symbol usages/definitions** (`search_symbol`) — semantic, not textual.
- **Type info at a location** (`get_symbol_info`) — without reading the file.
- **Compiler diagnostics** (`get_file_problems`) — structured IDE errors.
- **Macro expansion** (`generate_psi_tree`).
- **Cargo workspace structure** (`get_project_dependencies`, `get_project_modules`).

For everything else (string search, file reads, building, formatting) use Bash/Read.
