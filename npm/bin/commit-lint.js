#!/usr/bin/env node
import { spawn } from "node:child_process";
import { once } from "node:events";
import { createRequire } from "node:module";

const exe = process.platform === "win32" ? "commit-lint.exe" : "commit-lint";
const pkg = `@atypical/commit-${process.platform}-${process.arch}`;

let bin = process.env.COMMIT_LINT_BINARY;
if (!bin) {
  try {
    bin = createRequire(import.meta.url).resolve(`${pkg}/${exe}`);
  } catch {
    console.error(
      `commit-lint: ${pkg} is not installed ` +
        "(unsupported platform, or optional dependencies were skipped)",
    );
    process.exit(127);
  }
}

const child = spawn(bin, process.argv.slice(2), { stdio: "inherit" });
try {
  const [status] = await once(child, "exit");
  process.exit(status ?? 1);
} catch (error) {
  console.error(`commit-lint: ${error.message}`);
  process.exit(127);
}
