# Clawguard CLI Installation

## 1. Build From Source

Requirements:

- Rust toolchain
- `cargo`

Build the release binary:

```bash
cargo build --release -p clawguard
```

The binary will be available at:

```bash
target/release/clawguard
```

## 2. Run Directly From Source

```bash
cargo run -p clawguard -- help
```

## 3. Install Into Cargo Bin

```bash
cargo install --path crates/cli
```

This installs `clawguard` into Cargo's binary directory.

You can remove it later with:

```bash
cargo uninstall clawguard
```

## 4. Install With curl

Once release archives are published, users will be able to install with:

```bash
curl -fsSL https://raw.githubusercontent.com/legeling/ClawGuard/main/scripts/install-clawguard.sh | bash
```

Uninstall:

```bash
curl -fsSL https://raw.githubusercontent.com/legeling/ClawGuard/main/scripts/uninstall-clawguard.sh | bash
```

Optional environment variables:

- `CLAWGUARD_VERSION`
- `CLAWGUARD_INSTALL_DIR`
- `CLAWGUARD_REPO`

Example:

```bash
CLAWGUARD_VERSION=0.1.0 CLAWGUARD_INSTALL_DIR="$HOME/.local/bin" \
  curl -fsSL https://raw.githubusercontent.com/legeling/ClawGuard/main/scripts/install-clawguard.sh | bash
```

## 5. Install With npm or npx

Once the npm wrapper is published, users will be able to run:

```bash
npx clawguard --help
```

or:

```bash
npm install -g clawguard
clawguard --help
```

The npm wrapper lives under:

```text
packages/npm/clawguard
```

## 6. Local Packaging Workflow

Use the repository packaging script:

```bash
bash scripts/package-release.sh
```

Optional environment variables:

- `VERSION`: release version label
- `TARGET_TRIPLE`: target triple label for the archive name
- `OUTPUT_DIR`: packaging output directory

Example:

```bash
VERSION=0.1.0 TARGET_TRIPLE=aarch64-apple-darwin bash scripts/package-release.sh
```

## 7. Signed Rules-Pack Workflow

Generate a local signing keypair:

```bash
clawguard generate-signing-keypair --output-dir .keys
```

Sign the default rules into a versioned pack:

```bash
clawguard sign-rules-pack \
  --output rules-pack.json \
  --version 0.1.0 \
  --private-key .keys/clawguard-rules.private.key
```

Sign a custom rules file instead:

```bash
clawguard sign-rules-pack \
  --rules custom.rules \
  --output custom-pack.json \
  --version 0.2.0 \
  --private-key .keys/clawguard-rules.private.key \
  --key-id local-dev
```

Import and activate a signed rules pack:

```bash
clawguard import-rules-pack \
  --pack custom-pack.json \
  --public-key .keys/clawguard-rules.public.key \
  --activate
```

Inspect rules status:

```bash
clawguard rules-status
```

Roll back to the previous active rules version:

```bash
clawguard rollback-rules
```

Use the active rules store during scans:

```bash
clawguard scan --config example.conf --rules-store "$HOME/.clawguard/rules"
```

## 8. Post-Build Verification

Run the validation gate before shipping:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 9. Locale Usage

English report text:

```bash
clawguard scan --config example.conf --format text --locale en
```

Simplified Chinese report text:

```bash
clawguard scan --config example.conf --format text --locale zh-CN
```

Automatic locale detection:

- If `--locale` is omitted, ClawGuard checks `LC_ALL`, `LC_MESSAGES`, and `LANG`
- Chinese environments default to `zh-CN`
- Other environments default to English

## 10. Remaining Packaging Gaps

- Artifact signing is not implemented yet
- GitHub release archives must be published before `curl` and npm download flows work end-to-end
- Online rules-pack update checks are not implemented yet
- npm package is scaffolded but not published yet
