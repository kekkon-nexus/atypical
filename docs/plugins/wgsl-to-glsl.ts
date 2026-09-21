import { readFile } from "node:fs/promises";

import { parseWgsl, validate, writeGlsl } from "naga-wasm";
import type { Plugin } from "vite";

export function wgslToGlsl(): Plugin {
  return {
    name: "wgsl-to-glsl",
    enforce: "pre",
    async load(id) {
      if (!id.endsWith(".wgsl?glsl")) return;
      const file = id.slice(0, -"?glsl".length);
      this.addWatchFile(file);

      const content = await readFile(file, "utf8");
      using module = parseWgsl(content);
      using info = validate(module);

      const write = (stage: "vertex" | "fragment", entryPoint: string) =>
        writeGlsl(module, info, {
          version: "300 es",
          stage,
          entryPoint,
          flags: { adjustCoordinateSpace: false },
        });
      const vertex = write("vertex", "vs_main");
      const fragment = write("fragment", "fs_main");

      const block = /uniform (\w+) \{ \w+ (\w+); \};/.exec(fragment);
      if (!block) throw new Error(`${file}: no uniform block in the generated GLSL`);

      return [
        `export const vertex = ${JSON.stringify(vertex)};`,
        `export const fragment = ${JSON.stringify(fragment)};`,
        `export const uniformBlock = ${JSON.stringify(block[1])};`,
        `export const uniformStruct = ${JSON.stringify(block[2])};`,
      ].join("\n");
    },
  };
}
