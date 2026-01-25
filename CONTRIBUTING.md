# Contributing to Ritmo

Thank you for your interest in contributing to Ritmo! We welcome contributions from the community.

## Getting Started

1. **Fork the repository** and create your branch from `main`
2. **Clone your fork** locally
3. **Set up the development environment** (see below)

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/ritmo.git
cd ritmo

# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Run linter
cargo clippy --all -- -D warnings
```

## Branch Naming Convention

Use descriptive branch names:
- `feature/<description>` - For new features
- `fix/<description>` - For bug fixes
- `docs/<description>` - For documentation changes
- `refactor/<description>` - For code refactoring

## Code Standards

### Formatting
- Run `cargo fmt --all` before committing
- This project uses standard Rust formatting conventions

### Linting
- Run `cargo clippy --all -- -D warnings` and fix all warnings
- Clippy helps catch common mistakes and improve code quality

### Testing
- Add unit tests for new functionality
- Add integration tests where appropriate
- Ensure all tests pass before submitting a PR: `cargo test --workspace`
- Maintain or improve code coverage

### Documentation
- Document public APIs with doc comments (`///`)
- Update relevant documentation in `docs/` when needed
- If adding a new crate, update `docs/workspace.md`

## Commit Messages

- Write clear, concise commit messages
- Use imperative mood ("Add feature" not "Added feature")
- Reference issues when applicable (e.g., "Fix #123")
- Example format:
  ```
  Short summary (50 chars or less)

  More detailed explanation if needed. Wrap at 72 characters.

  Fixes #123
  ```

## Pull Request Process

1. **Update documentation** - Ensure docs reflect your changes
2. **Add tests** - All new functionality should have tests
3. **Run the full test suite** - `cargo test --workspace`
4. **Format and lint** - `cargo fmt --all && cargo clippy --all -- -D warnings`
5. **Write a clear PR description**:
   - What does this PR do?
   - Why is this change needed?
   - How to test it?
6. **Link related issues** - Reference any related issues

### PR Checklist

- [ ] Code follows project style guidelines
- [ ] Tests added/updated and passing
- [ ] Documentation updated (if needed)
- [ ] Commit messages are clear
- [ ] No merge conflicts with `main`
- [ ] `cargo fmt` and `cargo clippy` run without issues

## Adding New Crates

If you're adding a new crate to the workspace:

1. Create the crate: `cargo new --lib crate_name` or `cargo new --bin crate_name`
2. Add the path to `members` in the root `Cargo.toml`
3. Document the crate's purpose in `docs/workspace.md`
4. Add appropriate dependencies to the crate's `Cargo.toml`

## Security

- **Never commit secrets** - Use `.env.example` for example configurations
- **Never commit sensitive data** - No API keys, passwords, or tokens
- If you find a security vulnerability, please report it privately to the maintainers

## Code Review

- Maintainers will review your PR and may request changes
- Be open to feedback and discussion
- Reviews focus on code quality, architecture, and maintainability
- Address review comments in a timely manner

## Questions?

If you have questions about contributing:
- Check existing documentation in `docs/`
- Open an issue for discussion
- Reach out to maintainers

## License

By contributing to Ritmo, you agree that your contributions will be licensed under the same license as the project.

---

Thank you for contributing to Ritmo!
