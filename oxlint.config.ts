import oxlint from "@kekkon-nexus/config/oxlint";
import jsdoc from "@kekkon-nexus/config/oxlint/base/jsdoc";
import { defineConfig } from "oxlint";

export default defineConfig({
  extends: [oxlint],
  env: {
    node: true,
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
});
