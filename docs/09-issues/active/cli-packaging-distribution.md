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
- Release packaging: pending
- Installation workflow: pending

## Next Actions

1. Define release artifact targets for macOS, Linux, and Windows
2. Add installation instructions and a release checklist
3. Add sample packaged rules and report templates
4. Add a packaging-specific milestone to the changelog and state tracker
