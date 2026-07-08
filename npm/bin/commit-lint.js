#!/usr/bin/env node
import { spawnSync } from "node:child_process";
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

const { status, error } = spawnSync(bin, process.argv.slice(2), {
  stdio: "inherit",
});
if (error) {
  console.error(`commit-lint: ${error.message}`);
  process.exit(127);
}
process.exit(status ?? 1);
