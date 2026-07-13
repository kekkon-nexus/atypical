import base from "@kekkon-nexus/config/oxfmt";
import { defineConfig } from "oxfmt";

const config: ReturnType<typeof defineConfig> = defineConfig({
  ...base,
});

export default config;
