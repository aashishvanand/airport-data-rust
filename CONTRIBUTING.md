# Contributing to airport-data

Thank you for your interest in contributing to airport-data! This guide will help you get started.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- Git

### Setup

```bash
git clone https://github.com/aashishvanand/airport-data-rust.git
cd airport-data-rust
cargo build
```

### Running Tests

```bash
cargo test
```

### Running Lints

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

## Development Workflow

1. Fork the repository
2. Create a feature branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. Make your changes
4. Ensure all checks pass:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```
5. Commit your changes with a clear message:
   ```bash
   git commit -m "feat: add support for ..."
   ```
6. Push your branch and open a pull request against `main`

## Commit Message Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` — new feature
- `fix:` — bug fix
- `docs:` — documentation changes
- `style:` — formatting, no code change
- `refactor:` — code restructuring, no behavior change
- `test:` — adding or updating tests
- `chore:` — maintenance tasks (deps, CI, etc.)

## Branch Strategy

| Branch | Purpose |
|--------|---------|
| `main` | Development branch. All PRs merge here. Pushes trigger CI (fmt, clippy, tests). |
| `release` | Production branch. Merging `main` into `release` triggers CI + auto-publish to crates.io. |

## Updating Airport Data

If you want to update the airport dataset (`data/airports.json`):

1. Replace `data/airports.json` with the updated dataset
2. Ensure the JSON structure matches the existing schema (see `Airport` struct in `src/lib.rs`)
3. Run `cargo test` to verify all tests still pass
4. Submit a pull request with a description of what changed in the data

## Publishing a New Version

Publishing is handled automatically via CI/CD. Only maintainers can publish.

### Steps

1. Bump the version in `Cargo.toml` following [semver](https://semver.org/):
   - **Patch** (`1.0.x`) — bug fixes, data updates
   - **Minor** (`1.x.0`) — new features, backward-compatible
   - **Major** (`x.0.0`) — breaking API changes
2. Commit the version bump to `main`:
   ```bash
   # Edit Cargo.toml version
   cargo check  # updates Cargo.lock
   git add Cargo.toml Cargo.lock
   git commit -m "chore: bump version to X.Y.Z"
   git push origin main
   ```
3. Merge `main` into `release`:
   ```bash
   git checkout release
   git merge main
   git push origin release
   ```
4. The push to `release` triggers:
   - CI checks (fmt, clippy, tests on Linux/macOS/Windows)
   - Auto-publish to [crates.io](https://crates.io/crates/airport-data)
5. Switch back to `main`:
   ```bash
   git checkout main
   ```

### Requirements for Publishing

- The `CARGO_REGISTRY_TOKEN` secret must be configured in GitHub repository settings
- All CI checks must pass before the publish step runs

## Code Style

- Run `cargo fmt` before committing
- No clippy warnings (enforced with `-D warnings`)
- Add doc comments (`///`) to all public types and methods
- Add unit tests for new functionality

## Reporting Issues

- Use [GitHub Issues](https://github.com/aashishvanand/airport-data-rust/issues) to report bugs or request features
- Include Rust version (`rustc --version`) and OS when reporting bugs
- Provide a minimal reproduction if possible

## License

By contributing, you agree that your contributions will be licensed under the [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/) license.
