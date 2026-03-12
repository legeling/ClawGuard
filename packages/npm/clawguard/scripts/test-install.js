#!/usr/bin/env node

const assert = require("node:assert/strict");
const { detectPlatform, releaseUrl } = require("./install-lib");

const platform = detectPlatform();
assert.match(platform, /^(x86_64|aarch64)-/);

const release = releaseUrl("0.1.0", platform);
assert.equal(release.version, "0.1.0");
assert.match(release.archiveUrl, /^https:\/\/github\.com\//);
assert.ok(release.packageName.startsWith("clawguard-0.1.0-"));

console.log("npm wrapper install logic looks valid");
