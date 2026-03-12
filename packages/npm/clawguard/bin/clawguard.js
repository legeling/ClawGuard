#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const { existsSync } = require("node:fs");
const { join } = require("node:path");

const binaryName = process.platform === "win32" ? "clawguard.exe" : "clawguard";
const binaryPath = join(__dirname, "..", "vendor", binaryName);

if (!existsSync(binaryPath)) {
  console.error("ClawGuard binary is not installed yet.");
  console.error("Reinstall the package or run the postinstall script again.");
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit"
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
