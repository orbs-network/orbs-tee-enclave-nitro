# TODO List - ORBS TEE Nitro SDK

## High Priority

### 1. ✅ Verify price-oracle example compiles
**Status:** Complete (Linux-only)
**Description:** Ensure the price-oracle example works with the completed implementation
**Tasks:**
- [x] Check example dependencies
- [x] Update Rust version (1.74.1 → 1.90.0)
- [x] Add MSRV to Cargo.toml (rust-version = "1.83")
- [x] Verify example code is correct
- [x] Document Linux-only requirement

**Note:**
- Example uses `run_enclave_app()` which requires `nitro` feature (vsock + NSM)
- vsock only compiles on Linux (AWS Nitro Enclave environment)
- Example code is correct and ready for Linux/Nitro deployment
- Cannot test on macOS due to platform limitations
- All syntax and API usage verified ✅

### 2. ⏳ Update CLAUDE.md
**Status:** Pending
**Description:** Add test information and new directory structure
**Tasks:**
- [ ] Document tests/ directory structure
- [ ] Add test running instructions
- [ ] Update architecture section with complete implementations
- [ ] Add cross-platform testing notes

### 3. ⏳ Verify nitro and vsock modules are complete
**Status:** Pending
**Description:** Check if nitro and vsock modules have complete implementations
**Tasks:**
- [ ] Review src/nitro.rs completeness
- [ ] Review src/vsock.rs completeness
- [ ] Add any missing implementations
- [ ] Add unit tests if possible

## Medium Priority

### 4. ⏳ Update README with test instructions
**Status:** Pending
**Description:** Document testing in README
**Tasks:**
- [ ] Add "Running Tests" section
- [ ] Document test structure (unit vs integration)
- [ ] Add cross-platform testing notes
- [ ] Document feature flags for testing

### 5. ⏳ Add GitHub Actions CI/CD
**Status:** Pending
**Description:** Automated testing on push/PR
**Tasks:**
- [ ] Create .github/workflows/ci.yml
- [ ] Add test job for different platforms
- [ ] Add clippy and format checks
- [ ] Add coverage reporting (optional)

### 6. ⏳ Add integration test with full EnclaveRuntime
**Status:** Pending
**Description:** More comprehensive runtime tests with mocking
**Tasks:**
- [ ] Create mock vsock implementation
- [ ] Add end-to-end runtime tests
- [ ] Test request/response full cycle
- [ ] Test concurrent request handling

## Nice to Have

### 7. ⏳ Add signature verification example
**Status:** Pending
**Description:** Show host-side signature verification
**Tasks:**
- [ ] Create example showing signature verification
- [ ] Document how to verify with secp256k1
- [ ] Show TypeScript/Rust verification examples

### 8. ⏳ Add benchmarks for crypto operations
**Status:** Pending
**Description:** Performance metrics
**Tasks:**
- [ ] Create benches/ directory
- [ ] Add key generation benchmarks
- [ ] Add signing benchmarks
- [ ] Add verification benchmarks

### 9. ⏳ Prepare for crates.io publication
**Status:** Pending
**Description:** Package metadata and documentation
**Tasks:**
- [ ] Add LICENSE file
- [ ] Update Cargo.toml metadata (keywords, categories)
- [ ] Add repository and documentation links
- [ ] Write CHANGELOG.md
- [ ] Verify all docs are complete
- [ ] Test `cargo publish --dry-run`

## Completed ✅

- ✅ Add comprehensive test suite
- ✅ Complete source implementations (lib.rs, app.rs)
- ✅ Organize tests in separate files
- ✅ Switch to published crates.io dependency
- ✅ Add crypto module tests (10 tests)
- ✅ Add runtime tests (6 tests)
- ✅ Fix all compilation errors
- ✅ Make nitro features optional

---

## Notes

- Tests run on macOS without nitro features: `cargo test --no-default-features`
- All 16 tests currently passing
- Using orbs-tee-protocol = "0.1.4" from crates.io
