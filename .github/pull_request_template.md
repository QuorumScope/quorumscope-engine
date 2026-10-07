## Description

Provide a clear and concise summary of the changes made and the motivation behind them.

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Refactor or testing improvement

## Verification Checklist

- [ ] `cargo fmt --all -- --check` passes cleanly.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` reports no warnings.
- [ ] `cargo test --workspace --all-features` passes all tests.
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` builds without errors.
- [ ] All commits follow the Conventional Commits specification.
- [ ] Each commit represents a single logical change.
- [ ] No private keys, credentials, or sensitive data are included.
- [ ] Relevant documentation or OpenAPI schemas have been updated.
