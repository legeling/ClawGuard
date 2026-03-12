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

## 4. Local Packaging Workflow

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

## 5. Post-Build Verification

Run the validation gate before shipping:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 6. Locale Usage

English report text:

```bash
clawguard scan --config example.conf --format text --locale en
```

Simplified Chinese report text:

```bash
clawguard scan --config example.conf --format text --locale zh-CN
```

## 7. Remaining Packaging Gaps

- Artifact signing is not implemented yet
- Rules-pack signature verification distribution is not finalized yet
- Cross-platform install packages are not published yet
