# ClawGuard CI/CD

## CI

The repository validates the following on every push to `main` and on every pull request:

- `cargo build --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- shell-script syntax checks
- npm wrapper validation
- local release package build validation

Workflow:

- `.github/workflows/ci.yml`

## GitHub Release CD

Tagging a release with `v*` triggers cross-platform release packaging:

- `x86_64-unknown-linux-gnu`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Artifacts are uploaded and attached to the GitHub Release as `.tar.gz` archives.
When `RELEASE_SIGNING_PRIVATE_KEY` is configured, each archive is accompanied by a signed `.sig` manifest.

Workflow:

- `.github/workflows/release.yml`

## npm Publish CD

Publishing a GitHub Release or triggering the workflow manually can publish the npm wrapper package:

- package path: `packages/npm/clawguard`
- required secret: `NPM_TOKEN`

Workflow:

- `.github/workflows/npm-publish.yml`

## Required Repository Secrets

- `NPM_TOKEN`: npm publish token
- `RELEASE_SIGNING_PRIVATE_KEY`: Ed25519 PKCS#8 private key that matches `keys/release-public.pem`

## Release Checklist

1. Ensure CI is green
2. Create and push a tag like `v0.1.2`
3. Confirm `.tar.gz` and `.sig` release artifacts are attached
4. Confirm the install flows can verify the signed manifest against `keys/release-public.pem`
5. Confirm npm publish succeeded if npm distribution is desired
