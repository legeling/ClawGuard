const { createReadStream, createWriteStream, copyFileSync, mkdirSync, readFileSync, rmSync } = require("node:fs");
const { chmodSync } = require("node:fs");
const { createHash, verify } = require("node:crypto");
const { basename, join } = require("node:path");
const { get } = require("node:https");
const { spawnSync } = require("node:child_process");

const REPO_SLUG = process.env.CLAWGUARD_NPM_REPO || "legeling/ClawGuard";
const RELEASE_PUBLIC_KEY_URL =
  process.env.CLAWGUARD_RELEASE_PUBLIC_KEY_URL ||
  `https://raw.githubusercontent.com/${REPO_SLUG}/main/keys/release-public.pem`;

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
    archiveUrl: `https://github.com/${REPO_SLUG}/releases/download/v${normalizedVersion}/${packageName}.tar.gz`,
    signatureUrl: `https://github.com/${REPO_SLUG}/releases/download/v${normalizedVersion}/${packageName}.sig`,
    publicKeyUrl: RELEASE_PUBLIC_KEY_URL
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

function parseSignatureManifest(input) {
  const manifest = {};
  for (const rawLine of String(input).split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line) {
      continue;
    }

    const separatorIndex = line.indexOf("=");
    if (separatorIndex === -1) {
      throw new Error(`Invalid signature manifest line: ${line}`);
    }

    const key = line.slice(0, separatorIndex);
    const value = line.slice(separatorIndex + 1);
    manifest[key] = value;
  }

  return manifest;
}

function createVerificationPayload({ version, packageName, archiveName, sha256, keyId }) {
  return [
    `version=${version}`,
    `package_name=${packageName}`,
    `archive_name=${archiveName}`,
    `sha256=${sha256}`,
    `key_id=${keyId}`
  ].join("\n");
}

function verifySignatureManifest({
  manifest,
  expectedVersion,
  expectedPackageName,
  expectedArchiveName,
  publicKeyPem
}) {
  if (manifest.version !== expectedVersion) {
    throw new Error("Signed manifest version does not match the requested version");
  }

  if (manifest.package_name !== expectedPackageName) {
    throw new Error("Signed manifest package name does not match the downloaded artifact");
  }

  if (manifest.archive_name !== expectedArchiveName) {
    throw new Error("Signed manifest archive name does not match the downloaded artifact");
  }

  const payload = createVerificationPayload({
    version: manifest.version,
    packageName: manifest.package_name,
    archiveName: manifest.archive_name,
    sha256: manifest.sha256,
    keyId: manifest.key_id
  });

  const ok = verify(
    null,
    Buffer.from(payload),
    publicKeyPem,
    Buffer.from(manifest.signature, "base64")
  );

  if (!ok) {
    throw new Error("Signed manifest verification failed");
  }
}

function sha256File(filePath) {
  return new Promise((resolve, reject) => {
    const hash = createHash("sha256");
    const stream = createReadStream(filePath);

    stream.on("error", reject);
    stream.on("data", (chunk) => {
      hash.update(chunk);
    });
    stream.on("end", () => {
      resolve(hash.digest("hex"));
    });
  });
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
  const { packageName, archiveUrl, signatureUrl, publicKeyUrl } = releaseUrl(version, target);
  const tmpDir = join(packageRoot, ".tmp");
  const archivePath = join(tmpDir, `${packageName}.tar.gz`);
  const manifestPath = join(tmpDir, `${packageName}.sig`);
  const publicKeyPath = join(tmpDir, "release-public.pem");

  rmSync(tmpDir, { recursive: true, force: true });
  mkdirSync(tmpDir, { recursive: true });

  await download(archiveUrl, archivePath);
  await download(signatureUrl, manifestPath);
  await download(publicKeyUrl, publicKeyPath);

  const manifest = parseSignatureManifest(readFileSync(manifestPath, "utf8"));
  const actualSha256 = await sha256File(archivePath);
  if (manifest.sha256 !== actualSha256) {
    throw new Error("Downloaded archive checksum does not match the signed manifest");
  }
  verifySignatureManifest({
    manifest,
    expectedVersion: String(version).replace(/^v/, ""),
    expectedPackageName: packageName,
    expectedArchiveName: `${packageName}.tar.gz`,
    publicKeyPem: readFileSync(publicKeyPath, "utf8")
  });

  extract(archivePath, tmpDir);
  installBinary(packageRoot, packageName);
  rmSync(tmpDir, { recursive: true, force: true });

  return { target, archiveUrl, signatureUrl };
}

module.exports = {
  createVerificationPayload,
  detectPlatform,
  installFromRelease,
  parseSignatureManifest,
  releaseUrl
  ,
  verifySignatureManifest
};
