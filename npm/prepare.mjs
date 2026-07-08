// Usage: bun prepare.mjs <version>
//
// Materializes the platform packages under platforms/ from the GitHub
// release assets of v<version> (verifying each sha256), and stamps
// <version> into package.json. optionalDependencies are pinned by
// prepublish.mjs when publishing.
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import PLATFORMS from "./platforms.mjs";

const version = process.argv[2];
if (!version) {
  console.error("usage: bun prepare.mjs <version>");
  process.exit(2);
}

const root = path.dirname(fileURLToPath(import.meta.url));
const base = `https://github.com/kekkon-nexus/atypical/releases/download/v${version}`;

async function download(url) {
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`GET ${url} failed: ${res.status} ${res.statusText}`);
  }
  return Buffer.from(await res.arrayBuffer());
}

const main = JSON.parse(fs.readFileSync(path.join(root, "package.json")));
main.version = version;

for (const [platform, { target, os, cpu }] of Object.entries(PLATFORMS)) {
  const stem = `atypical-commit-v${version}-${target}`;
  const archive = `${stem}.${os === "win32" ? "zip" : "tar.gz"}`;
  const [data, checksum] = await Promise.all([
    download(`${base}/${archive}`),
    download(`${base}/${stem}.sha256`),
  ]);

  const expected = checksum.toString().trim().split(/\s+/)[0];
  const actual = crypto.createHash("sha256").update(data).digest("hex");
  if (actual !== expected) {
    throw new Error(`${archive}: checksum mismatch (${actual} != ${expected})`);
  }

  const name = `${main.name}-${platform}`;
  const dir = path.join(root, "platforms", platform);
  fs.mkdirSync(dir, { recursive: true });

  const file = path.join(dir, archive);
  fs.writeFileSync(file, data);
  try {
    const [cmd, ...args] =
      os === "win32" ? ["unzip", "-o", archive] : ["tar", "-xf", archive];
    execFileSync(cmd, args, { cwd: dir, stdio: "ignore" });
  } finally {
    fs.rmSync(file);
  }
  const exe = os === "win32" ? "commit-lint.exe" : "commit-lint";
  fs.chmodSync(path.join(dir, exe), 0o755);

  fs.writeFileSync(
    path.join(dir, "package.json"),
    JSON.stringify(
      {
        name,
        version,
        description: `${main.description} (${platform})`,
        license: main.license,
        contributors: main.contributors,
        repository: main.repository,
        os: [os],
        cpu: [cpu],
        publishConfig: main.publishConfig,
        files: [exe],
      },
      null,
      2,
    ) + "\n",
  );
  console.log(`${name}@${version} <- ${archive}`);
}

fs.writeFileSync(
  path.join(root, "package.json"),
  JSON.stringify(main, null, 2) + "\n",
);
