#!/usr/bin/env node

const assert = require("node:assert/strict");
const { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { generateKeyPairSync } = require("node:crypto");
const { spawnSync } = require("node:child_process");

const repoRoot = join(__dirname, "..");
const tempRoot = mkdtempSync(join(tmpdir(), "clawguard-package-test-"));
const outputDir = join(tempRoot, "artifacts");
const binaryPath = join(tempRoot, "clawguard");
const privateKeyPath = join(tempRoot, "release-private.pem");

mkdirSync(outputDir, { recursive: true });
writeFileSync(binaryPath, "stub-binary");

const { privateKey } = generateKeyPairSync("ed25519");
writeFileSync(privateKeyPath, privateKey.export({ type: "pkcs8", format: "pem" }));

const result = spawnSync("bash", ["scripts/package-release.sh"], {
  cwd: repoRoot,
  env: {
    ...process.env,
    VERSION: "0.0.0-test",
    TARGET_TRIPLE: "x86_64-unknown-linux-gnu",
    OUTPUT_DIR: outputDir,
    CLAWGUARD_SKIP_BUILD: "1",
    CLAWGUARD_BUILD_BINARY_PATH: binaryPath,
    CLAWGUARD_RELEASE_KEY_ID: "test",
    CLAWGUARD_RELEASE_PRIVATE_KEY_FILE: privateKeyPath
  },
  encoding: "utf8"
});

assert.equal(result.status, 0, result.stderr || result.stdout);

const packageName = "clawguard-0.0.0-test-x86_64-unknown-linux-gnu";
const manifestPath = join(outputDir, `${packageName}.sig`);
const manifestText = readFileSync(manifestPath, "utf8");
assert.match(manifestText, /sha256=/);
assert.match(manifestText, /signature=/);
assert.match(manifestText, /key_id=test/);

rmSync(tempRoot, { recursive: true, force: true });
console.log("release package signing looks valid");
