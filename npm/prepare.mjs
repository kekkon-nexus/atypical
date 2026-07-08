// Usage: bun prepare.mjs <version>
//
// Materializes the platform packages under platforms/ from the GitHub
// release assets of v<version> (verifying each sha256), and stamps
// <version> into package.json. optionalDependencies are pinned by
// prepublish.mjs when publishing.
import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import { execFile } from "node:child_process";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import PLATFORMS from "./platforms.mjs";

const version = process.argv[2];
if (!version) {
  console.error("usage: bun prepare.mjs <version>");
  process.exit(2);
}

const root = path.dirname(fileURLToPath(import.meta.url));
const base = `https://github.com/kekkon-nexus/atypical/releases/download/v${version}`;
const extract = promisify(execFile);

async function download(url) {
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`GET ${url} failed: ${res.status} ${res.statusText}`);
  }
  return Buffer.from(await res.arrayBuffer());
}

async function materialize(platform, { target, os, cpu, libc }) {
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
  await fs.mkdir(dir, { recursive: true });

  const file = path.join(dir, archive);
  await fs.writeFile(file, data);
  try {
    const [cmd, ...args] =
      os === "win32" ? ["unzip", "-o", archive] : ["tar", "-xf", archive];
    await extract(cmd, args, { cwd: dir });
  } finally {
    await fs.rm(file);
  }
  const exe = os === "win32" ? "commit-lint.exe" : "commit-lint";
  await fs.chmod(path.join(dir, exe), 0o755);

  await fs.writeFile(
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
        ...(libc && { libc: [libc] }),
        publishConfig: main.publishConfig,
        files: [exe],
      },
      null,
      2,
    ) + "\n",
  );
  console.log(`${name}@${version} <- ${archive}`);
}

const file = path.join(root, "package.json");
const main = JSON.parse(await fs.readFile(file));
main.version = version;

await Promise.all(
  Object.entries(PLATFORMS).map(([platform, spec]) =>
    materialize(platform, spec),
  ),
);

await fs.writeFile(file, JSON.stringify(main, null, 2) + "\n");
