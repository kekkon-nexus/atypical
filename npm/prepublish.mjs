// prepublishOnly: pin every platform package at exactly this version,
// so no publish path can ship a manifest without them. The committed
// package.json carries none (they only exist for published versions).
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import PLATFORMS from "./platforms.mjs";

const file = path.join(path.dirname(fileURLToPath(import.meta.url)), "package.json");
const main = JSON.parse(fs.readFileSync(file));

main.optionalDependencies = Object.fromEntries(
  Object.keys(PLATFORMS).map((platform) => [
    `${main.name}-${platform}`,
    main.version,
  ]),
);
fs.writeFileSync(file, JSON.stringify(main, null, 2) + "\n");
