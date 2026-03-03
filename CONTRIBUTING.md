# Contributing to TermiMon

Thanks for your interest in TermiMon! Here's how to get started.

## Development Setup

```bash
git clone https://github.com/ymatagne/termimon.git
cd termimon
cargo build
cargo test
```

**Requirements:** Rust 1.75+, tmux (for integration testing)

## Making Changes

1. Fork the repo
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run clippy: `cargo clippy -- -D warnings`
6. Format: `cargo fmt`
7. Commit with a descriptive message: `git commit -m 'feat: add new creature'`
8. Push and open a PR

## Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` — New feature
- `fix:` — Bug fix
- `docs:` — Documentation
- `refactor:` — Code restructuring
- `test:` — Adding tests
- `chore:` — Maintenance

## Adding Creatures

Creature sprites live in `src/creatures/`. Each creature needs:

1. A 16×16 pixel art sprite (3 evolution stages)
2. Color palette definition
3. Animation frames for each state (idle, typing, reading, thinking, sleeping, celebrating, error)
4. Agent detection pattern in `src/agents/`

See existing creatures for reference.

## Adding Agent Detectors

Agent detectors live in `src/agents/`. A detector needs:

1. Process name pattern matching
2. Optional output pattern matching
3. Creature assignment mapping

## Code Style

- Follow Rust idioms and `cargo clippy` suggestions
- Keep functions small and well-named
- Document public APIs with doc comments
- Error handling: use `anyhow` for application errors, `thiserror` for library errors

## Reporting Issues

- Use GitHub Issues
- Include: OS, terminal emulator, tmux version, termimon version
- For rendering issues: include a screenshot and terminal info (`echo $TERM`)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
