#!/usr/bin/env node

const assert = require("node:assert/strict");
const { generateKeyPairSync, sign } = require("node:crypto");
const { detectPlatform, releaseUrl } = require("./install-lib");
const {
  createVerificationPayload,
  parseSignatureManifest,
  verifySignatureManifest
} = require("./install-lib");

const platform = detectPlatform();
assert.match(platform, /^(x86_64|aarch64)-/);

const release = releaseUrl("0.1.2", platform);
assert.equal(release.version, "0.1.2");
assert.match(release.archiveUrl, /^https:\/\/github\.com\//);
assert.ok(release.packageName.startsWith("clawguard-0.1.2-"));
assert.match(release.signatureUrl, /^https:\/\/github\.com\//);

const manifestText = [
  "version=0.1.2",
  `package_name=${release.packageName}`,
  `archive_name=${release.packageName}.tar.gz`,
  "sha256=abc123",
  "key_id=test",
  "signature="
].join("\n");
const manifest = parseSignatureManifest(manifestText);
assert.equal(manifest.version, "0.1.2");
assert.equal(manifest.sha256, "abc123");

const { privateKey, publicKey } = generateKeyPairSync("ed25519");
const payload = createVerificationPayload({
  version: "0.1.2",
  packageName: release.packageName,
  archiveName: `${release.packageName}.tar.gz`,
  sha256: "abc123",
  keyId: "test"
});
const signature = sign(null, Buffer.from(payload), privateKey).toString("base64");
const signedManifest = parseSignatureManifest(`${manifestText}${signature}\n`);
verifySignatureManifest({
  manifest: signedManifest,
  expectedVersion: "0.1.2",
  expectedPackageName: release.packageName,
  expectedArchiveName: `${release.packageName}.tar.gz`,
  publicKeyPem: publicKey.export({ type: "spki", format: "pem" }).toString()
});

console.log("npm wrapper install logic looks valid");
