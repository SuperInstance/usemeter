# Publishing Checklist

## Pre-Publish

- [x] All 37 tests pass (100%)
- [x] Zero compiler warnings
- [x] Zero clippy warnings
- [x] README converts visitors in 10 seconds
- [x] All examples run without errors
- [ ] CI/CD passes on all platforms (Linux, macOS, Windows) - requires GitHub repo
- [x] Documentation complete (all public APIs documented)
- [ ] Cross-references added (SuperInstance uses this) - pending Round 2
- [x] LICENSE file present
- [x] CONTRIBUTING.md present

## Pre-Publish Verification

```bash
# Run all tests
cargo test --all-features

# Check for warnings
cargo clippy --all-features -- -D warnings

# Format check
cargo fmt -- --check

# Documentation build
cargo doc --all-features --no-deps

# Run examples (if any)
cargo run --example basic
```

## Test on crates.io

```bash
# Dry run (validates package without publishing)
cd /mnt/c/claudesuperinstance/privox
cargo publish --dry-run

# If dry run succeeds, publish for real
cargo publish
```

## Post-Publish

- [ ] Verify on crates.io: https://crates.io/crates/privox
- [ ] Verify docs.rs builds: https://docs.rs/privox
- [ ] Create GitHub release v0.1.0
- [ ] Update SuperInstance Tripartite1 to use published version
- [ ] Update ECOSYSTEM.md with privox

## Migration from synesis-privacy

Users migrating from `synesis-privacy` to `privox` need only two changes:

### 1. Update Cargo.toml

```toml
# Old
[dependencies]
synesis-privacy = "0.2"

# New
[dependencies]
privox = "0.1"
```

### 2. Update Imports

```rust
// Old
use synesis_privacy::{Redactor, TokenVault, PatternSet};

// New
use privox::{Redactor, TokenVault, PatternSet};
```

### 3. Update Code (Optional)

The API is 100% compatible. No code changes required beyond imports.

## Verification Steps

After publishing, verify the following:

1. **crates.io page loads**
   - Visit https://crates.io/crates/privox
   - Check metadata (description, keywords, categories)
   - Verify license is correct

2. **Documentation builds**
   - Visit https://docs.rs/privox
   - Check all public types are documented
   - Verify examples render correctly

3. **Dependencies resolve**
   - Create a new test project
   - Add `privox = "0.1"` to Cargo.toml
   - Run `cargo build`
   - Verify everything compiles

4. **SuperInstance integration**
   - Update SuperInstance's Cargo.toml
   - Run `cargo build --workspace`
   - Run `cargo test --workspace`
   - Verify all 250+ tests still pass

## Publishing Timeline

1. **Preparation** (Complete)
   - privox crate created
   - All tests passing
   - Documentation complete

2. **GitHub Setup** (Next)
   - Create repository: https://github.com/SuperInstance/privox
   - Push code
   - Create release v0.1.0

3. **crates.io Publishing** (Final)
   - Run `cargo publish --dry-run`
   - Fix any issues
   - Run `cargo publish`
   - Verify on crates.io and docs.rs

4. **SuperInstance Migration** (After publish)
   - Update Tripartite1 Cargo.toml
   - Update all imports
   - Verify tests pass
   - Commit and push

## Notes

- Do NOT publish to crates.io until orchestrator approves
- GitHub repo can be created anytime (private repo)
- SuperInstance can use path dependency during development
- Switch to crates.io dependency only after published
