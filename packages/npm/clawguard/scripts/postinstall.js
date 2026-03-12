#!/usr/bin/env node

const { join } = require("node:path");
const { installFromRelease } = require("./install-lib");

const version = process.env.CLAWGUARD_NPM_VERSION || require("../package.json").version;
const packageRoot = join(__dirname, "..");

if (process.env.CLAWGUARD_SKIP_DOWNLOAD === "1") {
  console.log("Skipping ClawGuard binary download because CLAWGUARD_SKIP_DOWNLOAD=1");
  process.exit(0);
}

installFromRelease({ packageRoot, version })
  .then(({ target, archiveUrl }) => {
    console.log(`Installed ClawGuard for ${target}`);
    console.log(`Source: ${archiveUrl}`);
  })
  .catch((error) => {
    console.error(`Failed to install ClawGuard: ${error.message}`);
    process.exit(1);
  });
