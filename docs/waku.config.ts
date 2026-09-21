import tailwindcss from "@tailwindcss/vite";
import { fumadocsMdx } from "fumadocs-mdx/vite";
import { defineConfig } from "waku/config";

import { wgslToGlsl } from "./plugins/wgsl-to-glsl";

export default defineConfig({
  vite: {
    resolve: {
      tsconfigPaths: true,
      dedupe: ["waku"],
    },

    plugins: [wgslToGlsl(), tailwindcss(), fumadocsMdx()],
  },
});
