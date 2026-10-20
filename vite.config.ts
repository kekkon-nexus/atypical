import oxfmt from "@kekkon-nexus/config/oxfmt";
import oxlint from "@kekkon-nexus/config/oxlint";
import jsdoc from "@kekkon-nexus/config/oxlint/base/jsdoc";
import vp from "@kekkon-nexus/config/oxlint/vite-plus";
import { defineConfig } from "vite-plus";

export default defineConfig({
  fmt: {
    ...oxfmt,
    ignorePatterns: ["*.toml", "benches/*.md"],
  },
  lint: {
    extends: [oxlint, vp],
    env: {
      node: true,
      browser: true,
    },
    options: {
      typeAware: true,
      typeCheck: true,
    },
    overrides: [
      {
        files: ["*.js"],
        ...jsdoc,
      },
    ],
  },
  staged: {
    "*.rs": () => [
      "cargo fmt --all",
      "cargo clippy --workspace --all-targets --all-features --allow-staged --allow-dirty --fix -- -D warnings",
    ],
    "*.toml": ["tombi format", "tombi lint"],
    "*.{json,yml,yaml}": (files) => {
      // v8r fails a batch whose every file .v8rignore drops
      const validated = files.filter((file) => !/docs\/content\/.*meta\.json$/.test(file));
      const v8r = validated.length > 0 ? [`v8r ${validated.join(" ")}`] : [];

      return [`vp fmt ${files.join(" ")}`, ...v8r];
    },
    "*.{js,jsx,ts,tsx,mjs,cjs}": "vp check --fix",
  },
});
