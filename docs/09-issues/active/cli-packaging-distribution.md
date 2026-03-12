# CLI Packaging and Distribution

## Summary

The current implementation is a stable Rust CLI MVP, but packaging and installation workflows are not finalized yet.

## Why This Matters

The product direction is now CLI-only. The remaining gap is distribution: a user should be able to install and run `clawguard` directly without building the workspace manually.

## Current State

- Rust core engine: implemented
- CLI entry point: implemented
- Ruleset loading: implemented
- Deployment profile scanning: implemented
- Localized report output: implemented
- Release packaging script: implemented
- Installation workflow documentation: implemented
- Artifact signing: pending
- Published install channels: pending

## Next Actions

1. Define release artifact targets for macOS, Linux, and Windows
2. Add artifact signing and verification workflow
3. Add published install channels and release notes process
4. Add a packaging-specific milestone to the changelog and state tracker
