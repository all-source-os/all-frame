# AllFrame Crate Dependency Graph & Publish Order

## Publish Order (leaves first)

1. `allframe-macros` — no workspace dependencies
2. `allframe-forge` — no workspace dependencies
3. `allframe-core` — depends on `allframe-macros` (optional)
4. `allframe-tauri` — depends on `allframe-core`
5. `allframe` — root crate, depends on `allframe-core` + `allframe-forge`

## Version Locations (all must be in sync)

In the root `Cargo.toml`:

| Line area | Field | Example |
|-----------|-------|---------|
| `[package]` | `version` | `"0.1.22"` |
| `[workspace.package]` | `version` | `"0.1.22"` |
| `[workspace.dependencies]` | `allframe-core` | `version = "0.1.22"` |
| `[workspace.dependencies]` | `allframe-forge` | `version = "0.1.22"` |
| `[workspace.dependencies]` | `allframe-macros` | `version = "0.1.22"` |
| `[workspace.dependencies]` | `allframe-tauri` | `version = "0.1.22"` |

All sub-crate Cargo.toml files use `version.workspace = true`, so they inherit automatically.

## CHANGELOG Format

Keep a Changelog format. Entry structure:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Fixed
### Added
### Changed
### Removed
### Migration
### Documentation
```

Include a `### Migration` section when there are breaking changes.
