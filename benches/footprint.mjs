// on-disk bytes of packages plus their transitive dependencies, each real file counted once
import { existsSync, readFileSync, readdirSync, realpathSync, statSync } from "node:fs";
import { dirname, join } from "node:path";

const seen = new Set();
let bytes = 0;

function locate(from, name) {
  for (let dir = from; dir !== dirname(dir); dir = dirname(dir)) {
    const hit = join(dir, "node_modules", name);
    if (existsSync(hit)) return realpathSync(hit);
  }
}

function tally(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === "node_modules") continue;
    const path = join(dir, entry.name);
    if (entry.isDirectory()) tally(path);
    else if (entry.isFile()) bytes += statSync(path).size;
  }
}

function walk(pkg) {
  if (!pkg || seen.has(pkg)) return;
  seen.add(pkg);
  tally(pkg);
  const manifest = JSON.parse(readFileSync(join(pkg, "package.json"), "utf8"));
  const deps = { ...manifest.dependencies, ...manifest.optionalDependencies };
  for (const name of Object.keys(deps)) walk(locate(pkg, name));
}

const [from, ...names] = process.argv.slice(2);
for (const name of names) walk(locate(realpathSync(from), name));
console.log(`${bytes} bytes, ${seen.size} packages`);
