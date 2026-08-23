# Upgrade and Migration Runbook

## Scope

This runbook covers safe upgrades of Resonique Server and migrations of stored collections, segments, WAL files, indexes, and configuration.

## Pre-upgrade checklist

- Confirm the target version and release notes.
- Record the current commit, Rust toolchain, configuration, and storage location.
- Stop writes and verify the service is healthy.
- Create and verify a tested backup of `data/`.
- Confirm sufficient disk space for backup and migration output.
- Run the current test and security-validation suites.
- Prepare a rollback version and rollback procedure.

## Upgrade procedure

1. Announce a maintenance window.
2. Stop the running server gracefully.
3. Verify no server process remains.
4. Back up the storage directory and configuration.
5. Build the target release.
6. Run database and configuration migrations, if required.
7. Start the target release using the existing configuration.
8. Verify health, metrics, collection loading, search, and WAL replay.
9. Run smoke tests against each configured collection.
10. Re-enable writes and monitor logs and resource usage.

## Storage migration procedure

- Never migrate the only copy of the data.
- Write migrated data to a separate destination.
- Preserve the source until validation and rollback periods expire.
- Validate record counts, metadata, indexes, segments, and search results.
- Test restart and WAL recovery using the migrated data.
- Record migration version and completion time.

## Rollback

1. Stop the target release.
2. Preserve target logs and failed migration output.
3. Restore the verified pre-upgrade backup.
4. Start the previous release.
5. Verify health, record counts, search behavior, and write capability.
6. Document the failure and keep the upgrade blocked until reviewed.

## Post-upgrade validation

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo audit
```

Repeat the complete validation suite a second time before marking the upgrade complete.

## Completion criteria

An upgrade is complete only when:

- Both validation runs pass.
- Health and metrics endpoints respond successfully.
- Existing collections load successfully.
- Insert and search smoke tests pass.
- Restart and WAL replay succeed.
- Backup and rollback artifacts are retained.
- Documentation and version records are updated.