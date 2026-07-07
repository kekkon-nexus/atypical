#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const path = require("node:path");

const exe = path.join(
  __dirname,
  process.platform === "win32" ? "commit-lint.exe" : "commit-lint",
);
const { status, error } = spawnSync(exe, process.argv.slice(2), {
  stdio: "inherit",
});
if (error) {
  console.error(
    error.code === "ENOENT"
      ? "commit-lint: binary missing; reinstall with scripts enabled (npm rebuild atypical-commit)"
      : `commit-lint: ${error.message}`,
  );
  process.exit(127);
}
process.exit(status ?? 1);
