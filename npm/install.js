// Postinstall: download the commit-lint binary for this platform from
// the GitHub release matching the package version, verify its sha256,
// and unpack it into bin/.
"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const { version } = require("./package.json");

const TARGETS = {
  "linux-x64": "x86_64-unknown-linux-gnu",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
  "win32-x64": "x86_64-pc-windows-msvc",
  "win32-arm64": "aarch64-pc-windows-msvc",
};

async function download(url) {
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`GET ${url} failed: ${res.status} ${res.statusText}`);
  }
  return Buffer.from(await res.arrayBuffer());
}

async function main() {
  const platform = `${process.platform}-${process.arch}`;
  const target = TARGETS[platform];
  if (!target) {
    throw new Error(
      `no prebuilt binary for ${platform}; use \`cargo install atypical-commit\` instead`,
    );
  }

  const stem = `atypical-commit-v${version}-${target}`;
  const archive = `${stem}.${process.platform === "win32" ? "zip" : "tar.gz"}`;
  const base = `https://github.com/kekkon-nexus/atypical/releases/download/v${version}`;

  const [data, checksum] = await Promise.all([
    download(`${base}/${archive}`),
    download(`${base}/${stem}.sha256`),
  ]);

  const expected = checksum.toString().trim().split(/\s+/)[0];
  const actual = crypto.createHash("sha256").update(data).digest("hex");
  if (actual !== expected) {
    throw new Error(`${archive}: checksum mismatch (${actual} != ${expected})`);
  }

  const dir = path.join(__dirname, "bin");
  const file = path.join(dir, archive);
  fs.writeFileSync(file, data);
  try {
    // Windows' tar.exe (bsdtar) also unpacks zip archives.
    execFileSync("tar", ["-xf", archive], { cwd: dir });
  } finally {
    fs.rmSync(file);
  }
  fs.chmodSync(
    path.join(dir, process.platform === "win32" ? "commit-lint.exe" : "commit-lint"),
    0o755,
  );
}

main().catch((err) => {
  console.error(`atypical-commit: ${err.message}`);
  process.exit(1);
});
