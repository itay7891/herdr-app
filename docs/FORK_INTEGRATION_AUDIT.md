# Herdr fork integration audit — 2026-11-07

Status: **BLOCKED for production deployment**. This is a source/documentation triage, **not** a complete security audit, successful build, or runtime test.

## P0: Core / GUI mismatch

The two repositories fork **different Herdr products**:

- `itay7891/herdr` is the legacy TUI `herdr` package, version 0.9.5 (`Cargo.toml`), with a terminal workspace-manager architecture.
- `itay7891/herdr-app` is the GPUI desktop client for the separate **`herdrdev/herdr-core`** project. Its README, AGENTS.md, and startup contract expect an `hc` executable and `hc connect` JSON-RPC protocol.

The existing `HARDENING.md` instructs users to build `itay7891/herdr` and set `HERDR_CORE` to `target/release/herdr`. **Do not follow this instruction**: the legacy TUI is not demonstrated to implement the `hc` protocol. Merely printing a version prefix is not a sufficient compatibility or authenticity check.

**Required before use:** fork and independently audit `herdrdev/herdr-core`; apply desired privacy controls there; pin an audited commit and build locally. Then verify `hc --version`, `hc connect` protocol handshake, remote SSH, and application integration in a disposable test account. Do not silently fall back to an installed binary.

## P0: Threat boundaries

- `herdr-app` mods run trusted JavaScript and can execute local/remote commands, read files, and access the network. `HERDR_MODS=0` is a useful temporary restriction, but disables important desktop UI components. It is **not** a replacement for OS sandboxing.
- The app's local control socket and app-home configuration are sensitive. Audit filesystem permissions, socket peer identity, symlinks, and local privilege assumptions on every target OS.
- Core access to SSH hosts, agent CLIs, project files, API credentials, and shell tools must be limited by OS identity and permissions. A GUI policy alone is not enforcement.
- Review submodules, dependency lockfiles, build scripts, CI workflows, and all outbound network paths before executing any build scripts or installers.
- Do not claim AgentSentry protects direct shell commands: MCP proxy enforcement covers only actions actually routed through it.

## P1: Multi-agent reliability

Before enabling autonomous handoffs, define durable task IDs, immutable handoff records, per-agent Git worktrees, explicit completion criteria, bounded retries, crash recovery, and human approval for merges/destructive operations. Verify the CLI/socket semantics against the actual audited core rather than assuming a shared protocol between legacy Herdr and hc.

## P1: AgentSentry integration test

Use synthetic repositories and non-sensitive test credentials. Cover ALLOW / BLOCK / REQUIRE_APPROVAL, denial of bypass through direct shell/network routes (or explicitly document uncovered paths), audit correlation IDs, agent identity, and replay prevention. Treat Herdr as an untrusted orchestration client, not an authorization authority.

## Release gate

1. Correct the core fork and build instructions.
2. Pin dependency/submodule revisions and inspect supply-chain risk.
3. Run static checks, unit/integration tests, and macOS + Ubuntu runtime smoke tests.
4. Capture network egress and audit socket/SSH permissions.
5. Test agent-to-agent handoff with isolated worktrees.
6. Obtain independent security review before deployment on machines containing real projects/secrets.

No source-code behavior was modified by this audit. The branch exists to keep findings separate from `master`.
