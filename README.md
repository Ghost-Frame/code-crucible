# code-crucible

Protocol enforcer for AI coding agents. Gates agent work behind structured reasoning steps -- spec, hypothesis, verification, challenge -- so AI agents write better code.

## Why

AI coding agents are powerful but chaotic. They skip planning, don't consider edge cases, and rarely verify their own work. Code-crucible enforces a structured protocol:

1. **Spec the task** before writing code (acceptance criteria, edge cases, interface contracts)
2. **Consider approaches** and evaluate tradeoffs before committing to one
3. **Log hypotheses** when debugging, track outcomes, build error memory
4. **Verify** changes with real commands, not just "looks right"
5. **Challenge** code adversarially before calling it done

Every step produces a structured JSON artifact stored in a local SQLite database. The protocol creates an audit trail of reasoning -- not just code diffs, but *why* decisions were made.

## Install

```bash
cargo install code-crucible
```

Or build from source:

```bash
git clone https://github.com/Ghost-Frame/code-crucible
cd code-crucible
cargo build --release
```

## Usage

Code-crucible communicates via JSON files. Every command takes `--input` (JSON) and `--output` (JSON):

```bash
code-crucible spec-task --input spec.json --output result.json
```

### Commands

**Planning & Design**
- `spec-task` -- Create a task specification with acceptance criteria, edge cases, and interface contracts
- `consider-approaches` -- Evaluate multiple implementation approaches with pros/cons/scores
- `think` -- Structured deep reasoning with constraints and context
- `declare-unknowns` -- Document blocking and non-blocking unknowns before proceeding

**Debugging**
- `log-hypothesis` -- Record a bug hypothesis with confidence score
- `log-outcome` -- Document whether a hypothesis was correct, incorrect, or partial
- `recall-errors` -- Search past hypotheses and outcomes for pattern matching

**Verification**
- `verify` -- Run a command and validate exit code (no shell -- direct exec for security)
- `challenge-code` -- Adversarially review a file for security, performance, and correctness issues
- `session-diff` -- Show git diff stats for the current session

**Session Management**
- `checkpoint` -- Save session state to a named checkpoint (with git ref)
- `rollback` -- Restore a previous checkpoint
- `session-learn` -- Record a discovery for future sessions
- `session-recall` -- Search past session learnings

**Code Analysis**
- `repo-map` -- Generate AST-aware repository structure map (supports Rust, TypeScript, Python, Go, JS, C, JSON)
- `search-code` -- Full-text code search with tree-sitter AST awareness

### Input Format

Each command expects specific JSON fields. Example for `spec-task`:

```json
{
  "task_description": "Add rate limiting to the /api/auth endpoint",
  "task_type": "feature",
  "acceptance_criteria": [
    "Rate limit of 10 requests per minute per IP",
    "Returns 429 with Retry-After header when exceeded"
  ],
  "interface_contract": "POST /api/auth returns 429 when rate limited",
  "edge_cases": [
    "Multiple IPs behind a proxy",
    "Rate limit reset at minute boundary",
    "Concurrent requests at the limit boundary"
  ],
  "files_to_touch": ["src/middleware/rate_limit.rs", "src/routes/auth.rs"],
  "dependencies": "Redis for distributed rate counting"
}
```

Example for `log-hypothesis`:

```json
{
  "bug_description": "Auth tokens expire 5 minutes early",
  "hypothesis": "Clock skew between auth server and API server causes premature expiration",
  "confidence": 0.8
}
```

### Output Format

All commands produce JSON with a consistent structure:

```json
{
  "success": true,
  "id": "spec_a1b2c3d4",
  "message": "Spec created",
  "data": { ... }
}
```

## Skill Backend (Optional)

Code-crucible can optionally integrate with a skill evolution backend -- a REST API that tracks reusable patterns, debugging approaches, and verified solutions across sessions.

Enable with the `skill-backend` feature:

```bash
cargo install code-crucible --features skill-backend
```

Set environment variables:

```bash
export CRUCIBLE_SKILL_URL=http://your-skill-server:4200
export CRUCIBLE_SKILL_API_KEY=your-api-key
```

This unlocks 6 additional commands:

- `skill-search` -- Find skills relevant to your current task
- `skill-capture` -- Create a new skill from a workflow description
- `skill-record-exec` -- Record execution success/failure (builds trust scores)
- `skill-fix` -- Trigger fix evolution on a failing skill
- `skill-derive` -- Combine parent skills into a new derived skill
- `skill-lineage` -- Show a skill's evolution chain

Existing commands also gain skill awareness:
- `spec-task` automatically searches for relevant skills and includes them in output
- `verify` can record pass/fail against a skill when `skill_id` is provided
- `session-learn` can auto-capture discoveries as skills when `capture_as_skill: true`

Any server implementing the expected REST endpoints (`/skills/search`, `/skills/capture`, `/skills/{id}/execute`, `/skills/{id}/fix`, `/skills/derive`, `/skills/{id}/lineage`) will work as a skill backend.

## Integration with AI Agents

Code-crucible is designed to be called by AI coding agents (Claude Code, Cursor, Copilot, custom agents) as part of their workflow. The agent writes JSON input, calls the CLI, reads JSON output.

Example integration in an agent's system prompt:

```
Before writing any code, run code-crucible spec-task with:
- Task description
- At least 2 acceptance criteria
- At least 3 edge cases
- Interface contract
- Files you plan to touch

After implementation, run code-crucible verify with the test command.
Then run code-crucible challenge-code on each modified file.
```

## Database

All state is stored in `~/.code-crucible/crucible.db` (SQLite). Override with `--db path/to/db`.

Tables: `specs`, `hypotheses`, `checkpoints`, `session_learns`, `approaches`.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions welcome. Please open an issue first for significant changes.
