import { $, argv } from "bun";

const input = argv[2];
if (!input) throw new Error("usage: bun run bump <major|minor|patch|alpha|beta|rc|release|X.Y.Z>");

const status = await $`git status --porcelain`.text();
if (status.trim()) throw new Error("working directory not clean");

const root = await $`git rev-parse --show-toplevel`.text();
$.cwd(root.trim());

const levels = ["major", "minor", "patch", "alpha", "beta", "rc", "release"];
await (levels.includes(input)
  ? $`cargo set-version --workspace --bump ${input}`
  : $`cargo set-version --workspace ${input}`);

interface Metadata {
  packages: { name: string; version: string }[];
}

const metadata = (await $`cargo metadata --no-deps --format-version 1`.json()) as Metadata;
const version = metadata.packages.find((pkg) => pkg.name === "atypical-commit")?.version;
if (!version) throw new Error("atypical-commit not found in cargo metadata");

await $`bun pm pkg set version=${version} --cwd npm`;
await $`bun install`;
await $`git commit -am ${`release: v${version}`}`;
await $`git tag -m ${`release: v${version}`} v${version}`;
