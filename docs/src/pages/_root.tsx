import type { ReactNode } from "react";

import { Provider } from "@/components/provider";

import "@/styles/globals.css";

export default function RootElement({ children }: { children: ReactNode }) {
  return (
    <html
      lang="en"
      suppressHydrationWarning
    >
      <head>
        <meta charSet="utf-8" />
        <link
          rel="icon"
          href="/favicon.ico"
          sizes="32x32"
        />
        <link
          rel="icon"
          href="/favicon.svg"
          type="image/svg+xml"
        />
      </head>
      <body className="flex flex-col min-h-screen">
        <Provider>{children}</Provider>
      </body>
    </html>
  );
}

export function getConfig() {
  return {
    render: "static",
  } as const;
}
