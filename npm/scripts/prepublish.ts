import fs from "node:fs/promises";
import path from "node:path";

import main from "../package.json" with { type: "json" };
import platforms from "./platforms";

if (import.meta.main) {
  main.optionalDependencies = Object.fromEntries(
    Object.keys(platforms).map((platform) => [`${main.name}-${platform}`, main.version]),
  );

  const file = path.join(import.meta.dirname, "..", "package.json");
  // oxlint-disable-next-line prefer-template
  await fs.writeFile(file, JSON.stringify(main, undefined, 2) + "\n");
}
