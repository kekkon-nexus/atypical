import { RiFileSettingsLine, RiGitBranchLine, RiScalesLine, RiTerminalLine } from "@remixicon/react";
import { ServerCodeBlock } from "fumadocs-ui/components/codeblock.rsc";
import type { ReactNode } from "react";
import { Link } from "waku";

import Dither from "@/components/dither";
import { buttonVariants } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { appName } from "@/lib/shared";
import { cn } from "@/lib/utils";

const repository = "https://github.com/kekkon-nexus/atypical";

export default function Home() {
  return (
    <div className="pt-4 pb-6 md:pb-12">
      <title>{appName}</title>
      <div className="relative flex min-h-150 h-[70vh] max-h-225 border rounded-2xl overflow-hidden mx-auto w-full max-w-350 bg-origin-border">
        <div className="absolute inset-0">
          <Dither size={32} shapeScale={1.2} timeScale={0.5} className="opacity-33" />
        </div>
        <div className="flex flex-col z-2 px-4 size-full md:p-12 max-md:items-center max-md:text-center">
          <p className="mt-12 text-xs text-fd-primary font-medium p-2 border rounded-lg border-fd-primary/50 bg-fd-primary-foreground dark:bg-fd-primary-foreground/50 backdrop-blur-xs w-fit">
            (Non)standard enforcing DX.
          </p>
          <h1 className="text-4xl my-8 leading-tighter font-medium xl:text-5xl xl:mb-12">
            Enforce <br className="md:hidden" />
            <span className="text-fd-primary bg-fd-primary-foreground dark:bg-fd-primary-foreground/50 backdrop-blur-xs">
              your conventions
            </span>
            ,
            <br />
            not ours.
          </h1>
          <Actions />
        </div>
      </div>
      <div className="grid grid-cols-1 gap-10 mt-12 px-6 mx-auto w-full max-w-350 md:px-12 lg:grid-cols-2 lg:mt-20">
        <p className="text-2xl tracking-tight leading-snug font-light col-span-full md:text-3xl xl:text-4xl">
          Atypical is a <span className="text-fd-primary font-medium">toolkit</span> for enforcing
          your own conventions, configured from{" "}
          <span className="text-fd-primary font-medium">one file</span> in which each tool reads its
          own section. Bring your grammar, not someone else&apos;s.
        </p>
        <TryItOut />
        <Feature
          icon={<RiFileSettingsLine />}
          title="One file, every tool."
          description={
            <>
              Every tool finds the nearest <code>atypical.toml</code> and reads only its own
              section. Start from a preset, override what differs.
            </>
          }
        >
          <ServerCodeBlock
            code={`extends = "presets/standard.toml"`}
            lang="toml"
          />
        </Feature>
        <Feature
          title="Your grammar, declared."
          description="A commit header is a list of slots in header order: words, symbols, or delimited values."
        >
          <ServerCodeBlock
            codeblock={{ title: "atypical.toml" }}
            code={`[[commit.slots]]
name = "keywords"
kind = "word"
values = ["feat", "fix"]
required = true`}
            lang="toml"
          />
        </Feature>
        <SectionHeading>Hooks and CI.</SectionHeading>
        <Feature
          title="Catch it at commit."
          description="Drop it into a commit-msg hook, husky or plain git."
        >
          <ServerCodeBlock
            codeblock={{ title: ".husky/commit-msg" }}
            code={`commit-lint -- "$1"`}
            lang="bash"
          />
        </Feature>
        <Feature
          icon={<RiGitBranchLine />}
          title="Or the whole range."
          description="Lint every commit a pull request brings in."
        >
          <ServerCodeBlock
            code="commit-lint --from origin/main"
            lang="bash"
          />
        </Feature>
        <SectionHeading>Open Source.</SectionHeading>
        <Feature
          className="col-span-full"
          icon={<RiScalesLine />}
          title="MIT or Apache-2.0."
          description="At your option."
        >
          <Actions />
        </Feature>
      </div>
    </div>
  );
}

function Actions() {
  return (
    <div className="flex flex-row items-center gap-4 flex-wrap w-fit">
      <Link
        to="/docs"
        className={cn(buttonVariants({ size: "lg" }))}
      >
        Read docs
      </Link>
      <a
        href={repository}
        target="_blank"
        rel="noreferrer noopener"
        className={cn(buttonVariants({ size: "lg", variant: "outline" }), "backdrop-blur-xs")}
      >
        Open GitHub
      </a>
    </div>
  );
}

function SectionHeading({ children }: { children: ReactNode }) {
  return (
    <h2 className="mt-8 mb-4 col-span-full text-center text-3xl font-medium tracking-tight text-fd-primary lg:text-4xl">
      {children}
    </h2>
  );
}

function Feature({
  icon,
  title,
  description,
  className,
  children,
}: {
  icon?: ReactNode;
  title: ReactNode;
  description: ReactNode;
  className?: string;
  children: ReactNode;
}) {
  return (
    <Card className={className}>
      <CardHeader>
        {icon && <div className="text-fd-primary mb-2">{icon}</div>}
        <CardTitle className="text-xl font-medium tracking-tight lg:text-2xl">{title}</CardTitle>
        <CardDescription className="text-sm">{description}</CardDescription>
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}

function TryItOut() {
  return (
    <Card className="col-span-full mx-auto w-full max-w-200 p-2">
      <div className="flex flex-row gap-2">
        <h2 className="text-fd-primary content-center font-mono font-bold uppercase [text-box-trim:both] [text-box-edge:cap_alphabetic] border-2 border-fd-primary/50 px-2 rounded-xl">
          Try it out
        </h2>
        <ServerCodeBlock
          code="cargo install atypical-commit"
          lang="bash"
          codeblock={{ className: "bg-fd-secondary flex-1" }}
        />
      </div>
      <div className="bg-fd-secondary rounded-xl border shadow-md">
        <div className="flex flex-row items-center gap-2 border-b p-2 text-fd-muted-foreground">
          <RiTerminalLine className="size-4" />
          <span className="text-xs font-medium">Terminal</span>
          <div className="ms-auto me-2 size-2 rounded-full bg-red-400" />
        </div>
        <pre className="p-2 text-sm font-mono text-fd-secondary-foreground/80 whitespace-pre-wrap">
          {`$ echo 'add(lib)[int]: something' | commit-lint -
$ echo $?
0`}
        </pre>
      </div>
    </Card>
  );
}

export function getConfig() {
  return {
    render: "static",
  };
}
