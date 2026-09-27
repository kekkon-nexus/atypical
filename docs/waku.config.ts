import path from "node:path";

import tailwindcss from "@tailwindcss/vite";
import { fumadocsMdx } from "fumadocs-mdx/vite";
import { defineConfig } from "waku/config";

import { wgslToGlsl } from "./plugins/wgsl-to-glsl";

export default defineConfig({
  vite: {
    optimizeDeps: {
      exclude: ["@base-ui/react"],
    },
    resolve: {
      tsconfigPaths: true,
      dedupe: ["waku"],
      alias: { "lucide-react": path.resolve(import.meta.dirname, "src/lib/lucide-shim.ts") },
    },

    plugins: [fumadocsMdx(), tailwindcss(), wgslToGlsl()],
  },
});
