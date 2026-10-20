import { readFileSync } from "node:fs";
import path from "node:path";

import { gitConfig } from "./shared";

/**
 * The workspace version, read from the root `Cargo.toml` at build time.
 *
 * @returns The version and the GitHub release it is tagged as.
 */
export function release() {
  const manifest = readFileSync(path.resolve(process.cwd(), "../Cargo.toml"), "utf8");
  const version = /^\[workspace\.package\][^[]*?^version = "([^"]+)"/ms.exec(manifest)?.[1] ?? "";

  return {
    version,
    url: `https://github.com/${gitConfig.user}/${gitConfig.repo}/releases/tag/v${version}`,
  };
}
