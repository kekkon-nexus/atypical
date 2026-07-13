import { execFile } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import { promisify } from "node:util";

import platforms from "./platforms";

const version = process.argv.at(2);
if (!version) {
  // oxlint-disable-next-line no-console
  console.error("usage: bun scripts/prepare.ts <version>");
  // oxlint-disable-next-line unicorn/no-process-exit
  process.exit(2);
}

const root = import.meta.dirname;
const base = `https://github.com/kekkon-nexus/atypical/releases/download/v${version}`;
const extract = promisify(execFile);

async function download(url: string) {
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`GET ${url} failed: ${res.status} ${res.statusText}`);
  }
  return Buffer.from(await res.arrayBuffer());
}

async function materialize(
  platform: string,
  {
    target,
    os,
    cpu,
    libc,
  }: {
    target: string;
    os: string;
    cpu: string;
    libc?: string;
  },
) {
  const stem = `atypical-commit-v${version}-${target}`;
  const archive = `${stem}.${os === "win32" ? "zip" : "tar.gz"}`;
  const [data, checksum] = await Promise.all([
    download(`${base}/${archive}`),
    download(`${base}/${stem}.sha256`),
  ]);

  const [expected] = checksum.toString().trim().split(/\s+/);
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
    const [cmd, ...args] = os === "win32" ? ["unzip", "-o", archive] : ["tar", "-xf", archive];
    await extract(cmd, args, { cwd: dir });
  } finally {
    await fs.rm(file);
  }
  const exe = os === "win32" ? "commit-lint.exe" : "commit-lint";
  await fs.chmod(path.join(dir, exe), 0o755);

  await fs.writeFile(
    path.join(dir, "package.json"),
    // oxlint-disable-next-line prefer-template
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
  // oxlint-disable-next-line no-console
  console.log(`${name}@${version} <- ${archive}`);
}

const file = path.join(root, "package.json");
const main = JSON.parse(await fs.readFile(file, "utf8"));
main.version = version;

await Promise.all(Object.entries(platforms).map(([platform, spec]) => materialize(platform, spec)));

// oxlint-disable-next-line prefer-template
await fs.writeFile(file, JSON.stringify(main, null, 2) + "\n");
