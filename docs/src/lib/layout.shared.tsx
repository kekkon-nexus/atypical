import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

import { searchTrigger } from "@/components/search-trigger";
import { ThemeSwitch } from "@/components/theme-switch";

import { appName, gitConfig } from "./shared";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      // JSX supported
      title: appName,
    },
    slots: { searchTrigger, themeSwitch: ThemeSwitch },
    githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
  };
}
