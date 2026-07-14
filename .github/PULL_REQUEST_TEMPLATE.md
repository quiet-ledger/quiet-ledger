# Pull Request Checklist

Thank you for contributing! Please review the following checklist to ensure your PR is ready for review.

## Summary
- [ ] Provide a clear and concise description of the changes.
- [ ] Link to any related issues using `Fixes #issue` or `Related to #issue`.

## Testing
- [ ] All new code is covered by unit tests where applicable.
- [ ] Existing tests pass locally (`cargo test` for contracts/SDKs, `npm test` for the TS SDK) — see `CONTRIBUTING.md` for a known `cargo test` toolchain caveat if it fails to compile.
- [ ] Added tests for edge cases and error conditions.
- [ ] Updated `e2e/` scripts if you changed a contract's public interface.

## Documentation
- [ ] Updated `README.md` / `ARCHITECTURE.md` if changes affect users or the design.
- [ ] Updated `docs/RFC.md` if this changes the protocol or data model.
- [ ] Updated `docs/THREAT_MODEL.md` if this changes a trust assumption or what's disclosed on-chain.

## Code Quality
- [ ] Follows the project's coding style and conventions.
- [ ] No commented-out code or debug statements left in the codebase.
- [ ] Variables and functions are named descriptively.
- [ ] Code is properly formatted (`cargo fmt` / circuit files reviewed by hand).
- [ ] No new clippy warnings (`cargo clippy`).

## Breaking Changes
- [ ] If this PR introduces breaking changes (proof encoding, public-signal ordering, contract interfaces), describe them and provide migration steps.
- [ ] Updated version in `Cargo.toml` / `package.json` if appropriate (following semver).

## Additional Notes
- [ ] Any other relevant information for reviewers.

Please ensure all checkboxes are checked (or explicitly marked N/A with a reason) before requesting a review.
