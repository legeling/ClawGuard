const { createWriteStream, copyFileSync, mkdirSync, rmSync } = require("node:fs");
const { chmodSync } = require("node:fs");
const { basename, join } = require("node:path");
const { get } = require("node:https");
const { spawnSync } = require("node:child_process");

const REPO_SLUG = process.env.CLAWGUARD_NPM_REPO || "legeling/ClawGuard";

function detectPlatform() {
  const arch = process.arch === "x64" ? "x86_64" : process.arch === "arm64" ? "aarch64" : process.arch;
  if (!["x86_64", "aarch64"].includes(arch)) {
    throw new Error(`Unsupported architecture: ${process.arch}`);
  }

  if (process.platform === "darwin") {
    return `${arch}-apple-darwin`;
  }
  if (process.platform === "linux") {
    return `${arch}-unknown-linux-gnu`;
  }
  if (process.platform === "win32") {
    return `${arch}-pc-windows-msvc`;
  }

  throw new Error(`Unsupported platform: ${process.platform}`);
}

function releaseUrl(version, target) {
  const normalizedVersion = String(version).replace(/^v/, "");
  const packageName = `clawguard-${normalizedVersion}-${target}`;
  return {
    version: normalizedVersion,
    packageName,
    archiveUrl: `https://github.com/${REPO_SLUG}/releases/download/v${normalizedVersion}/${packageName}.tar.gz`
  };
}

function download(url, destination) {
  return new Promise((resolve, reject) => {
    const file = createWriteStream(destination);
    get(url, (response) => {
      if (response.statusCode !== 200) {
        reject(new Error(`Download failed with status ${response.statusCode}`));
        response.resume();
        return;
      }

      response.pipe(file);
      file.on("finish", () => {
        file.close(resolve);
      });
    }).on("error", reject);
  });
}

function extract(archivePath, destination) {
  mkdirSync(destination, { recursive: true });
  const result = spawnSync("tar", ["-xzf", archivePath, "-C", destination], {
    stdio: "inherit"
  });
  if (result.status !== 0) {
    throw new Error("Failed to extract archive with tar");
  }
}

function installBinary(packageRoot, extractedDirName) {
  const vendorDir = join(packageRoot, "vendor");
  const binaryName = process.platform === "win32" ? "clawguard.exe" : "clawguard";
  const sourcePath = join(packageRoot, ".tmp", extractedDirName, binaryName);
  const targetPath = join(vendorDir, binaryName);

  mkdirSync(vendorDir, { recursive: true });
  copyFileSync(sourcePath, targetPath);

  if (process.platform !== "win32") {
    chmodSync(targetPath, 0o755);
  }
}

async function installFromRelease({ packageRoot, version }) {
  const target = detectPlatform();
  const { packageName, archiveUrl } = releaseUrl(version, target);
  const tmpDir = join(packageRoot, ".tmp");
  const archivePath = join(tmpDir, `${packageName}.tar.gz`);

  rmSync(tmpDir, { recursive: true, force: true });
  mkdirSync(tmpDir, { recursive: true });

  await download(archiveUrl, archivePath);
  extract(archivePath, tmpDir);
  installBinary(packageRoot, packageName);
  rmSync(tmpDir, { recursive: true, force: true });

  return { target, archiveUrl };
}

module.exports = {
  detectPlatform,
  installFromRelease,
  releaseUrl
};
