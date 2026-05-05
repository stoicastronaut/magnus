# Contributing to Magnus

Thanks for helping improve Magnus. This project is pre-release, so small, focused contributions are the easiest to review and merge.

## Contribution Flow

1. Open an issue or discussion for large changes, architecture changes, security-sensitive work, or product behavior that is not already agreed.
2. Fork the repository or create a feature branch.
3. Keep the pull request focused on one behavior or cleanup.
4. Include tests when the change affects behavior.
5. Make sure CI passes before the PR can be merged.

Small fixes, documentation improvements, and typo fixes can go straight to a pull request.

## Local Setup

Requirements:

- Rust `1.95.0`
- Node.js 25
- pnpm `10.25.0`
- Tauri system dependencies for your operating system

Install dependencies:

```bash
pnpm install --frozen-lockfile
```

Run the app:

```bash
cargo tauri dev
```

## Checks

CI is authoritative and must pass before merge. Contributors are encouraged to run the relevant checks locally before opening a PR:

```bash
pnpm tsc --noEmit
pnpm lint
pnpm test
cargo fmt --manifest-path backend/Cargo.toml -- --check
cargo clippy --manifest-path backend/Cargo.toml -- -D warnings
cargo test --manifest-path backend/Cargo.toml
```

Coverage gates also run in CI:

```bash
pnpm test:coverage
cargo llvm-cov --manifest-path backend/Cargo.toml --lib --fail-under-lines 90
```

The project may temporarily be below the target coverage threshold while missing tests are addressed in follow-up work.

## Project Conventions

- Use the domain language in `CONTEXT.md`.
- Check `docs/adr/` before changing hard-to-reverse architecture decisions.
- Keep React components focused and small.
- Keep provider API calls in the Rust backend.
- Keep Provider API Keys out of App Data, diagnostics exports, screenshots, and test fixtures.
- Do not introduce a separate network proxy concept for Custom Providers.

## Privacy and Secrets

Do not share Provider API Keys, real Chat Transcripts, unredacted Diagnostics Exports, or private endpoint URLs in issues, pull requests, screenshots, logs, or tests.

Use fake data in examples and tests. If you need to report a bug with sensitive diagnostic information, redact it first or use the security reporting path below.

## Security Reports

Please do not file public issues for suspected vulnerabilities, secret leakage, unsafe diagnostics exports, or System Secret Store problems.

For now, contact the maintainer privately before sharing details:

- Security contact: `TODO: add security contact`

If GitHub private vulnerability reporting is enabled later, use the repository Security tab instead.

## Pull Request Review

Maintainers review for:

- Correct behavior.
- Passing CI.
- Clear scope.
- Tests for behavior changes.
- No accidental secrets or private data.
- Alignment with the architecture documented in `CONTEXT.md` and `docs/adr/`.

PRs may be asked to split large changes into smaller pieces before merge.
