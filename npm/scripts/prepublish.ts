import fs from "node:fs/promises";
import path from "node:path";

import platforms from "./platforms";

const file = path.join(import.meta.dirname, "package.json");
const main = JSON.parse(await fs.readFile(file, "utf8"));

main.optionalDependencies = Object.fromEntries(
  Object.keys(platforms).map((platform) => [`${main.name}-${platform}`, main.version]),
);

// oxlint-disable-next-line prefer-template
await fs.writeFile(file, JSON.stringify(main, null, 2) + "\n");
