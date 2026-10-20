import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

import { searchTrigger } from "@/components/search-trigger";
import { ThemeSwitch } from "@/components/theme-switch";

import { appName, gitConfig } from "./shared";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <span className="rounded-md bg-[#3d6300] px-1.5 py-0.5 font-medium text-[#9ae600]">{appName}</span>
      ),
    },
    slots: { searchTrigger, themeSwitch: ThemeSwitch },
    githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
  };
}
