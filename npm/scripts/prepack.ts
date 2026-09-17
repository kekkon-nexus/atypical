import fs from "node:fs/promises";
import path from "node:path";

if (import.meta.main) {
  const root = path.join(import.meta.dirname, "..");
  const presets = path.join(root, "presets");

  await fs.rm(presets, { recursive: true, force: true });
  await fs.cp(path.join(root, "..", "presets"), presets, { recursive: true });
}
