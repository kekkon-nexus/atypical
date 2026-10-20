import {
  RiCheckLine,
  RiCloseLine,
  RiFileSettingsLine,
  RiGitBranchLine,
  RiScalesLine,
  RiTerminalLine,
} from "@remixicon/react";
import { ServerCodeBlock } from "fumadocs-ui/components/codeblock.rsc";
import type { ReactNode } from "react";
import { Link } from "waku";

import Dither from "@/components/dither";
import { BenchRace, Switcher, Terminal } from "@/components/home/client";
import { buttonVariants } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { floorLeading, latestBench } from "@/lib/bench";
import { release } from "@/lib/release";
import { appName } from "@/lib/shared";
import { cn } from "@/lib/utils";

const repository = "https://github.com/kekkon-nexus/atypical";

const presets = [
  {
    value: "standard",
    label: "Standard Commits",
    headers: [
      ["add(exe)[int]: initial", true],
      ["ref!![eff]: x", true],
      ["fix!(ci)[sec]: x", true],
      ["feat: x", false],
      ["add(exe)!: x", false],
    ],
  },
  {
    value: "conventional",
    label: "Conventional Commits",
    headers: [
      ["feat: an endpoint", true],
      ["fix(parser): handle empty input", true],
      ["feat(api)!: drop the v1 routes", true],
      ["feat!(api): x", false],
      ["feat[api]: x", false],
    ],
  },
  {
    value: "gitmoji",
    label: "Gitmoji",
    headers: [
      ["✨ Add a feature", true],
      [":sparkles: Add a feature", true],
      ["🐛 (auth): Fix a bug", true],
      ["feat: no intention", false],
      ["✨(auth): x", false],
    ],
  },
] satisfies { value: string; label: string; headers: [string, boolean][] }[];

export default function Home() {
  const bench = latestBench();
  const { version, url } = release();

  return (
    <div className="pt-4 pb-6 md:pb-12">
      <title>{appName}</title>
      <div className="relative border rounded-2xl overflow-hidden mx-auto w-full max-w-350 bg-origin-border">
        <div className="absolute inset-0">
          <Dither size={32} shapeScale={1.2} timeScale={0.5} className="opacity-[0.125]" />
        </div>
        <div className="relative z-2 grid grid-cols-1 gap-10 p-4 md:px-12 md:py-16 lg:min-h-176 lg:grid-cols-2 lg:items-center">
          <div className="flex flex-col">
            <a
              href={url}
              className="mt-8 text-xs text-fd-primary font-mono font-medium uppercase p-2 border rounded-lg border-fd-primary/50 bg-fd-primary-foreground dark:bg-fd-primary-foreground/50 backdrop-blur-xs w-fit"
            >
              v{version} released →
            </a>
            <h1 className="text-4xl my-8 leading-tighter font-medium xl:text-5xl">
              Enforce{" "}
              <span className="text-fd-primary bg-fd-primary-foreground dark:bg-fd-primary-foreground/50 backdrop-blur-xs">
                your conventions
              </span>
              ,
              <br />
              not ours.
            </h1>
            <p className="max-w-120 text-lg text-fd-muted-foreground">
              A commit message linter configured from one <code>atypical.toml</code>. A native
              binary, no Node.js runtime.
            </p>
            <p className="mt-8 mb-2 text-sm font-medium">Install atypical v{version}</p>
            <Switcher
              className="flex max-w-120 flex-col-reverse gap-2"
              options={[
                {
                  value: "npm",
                  label: "npm",
                  panel: (
                    <ServerCodeBlock
                      code="npm i -D @atypical/commit"
                      lang="bash"
                    />
                  ),
                },
                {
                  value: "cargo",
                  label: "cargo",
                  panel: (
                    <ServerCodeBlock
                      code="cargo install atypical-commit"
                      lang="bash"
                    />
                  ),
                },
              ]}
            />
            <Link
              to="/docs"
              className="mt-6 w-fit font-medium underline underline-offset-4"
            >
              Then read the docs →
            </Link>
          </div>
          {bench && (
            <div className="flex flex-col gap-4">
              <p className="text-2xl font-medium tracking-tight text-fd-primary lg:text-3xl">
                +{floorLeading(bench.speedup)}× faster.{" "}
                {floorLeading(bench.size.commitlint / bench.size.atypical)}× smaller.
              </p>
              <Card className="p-6">
                <BenchRace bench={bench} />
              </Card>
            </div>
          )}
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
            code={`extends = "npm:@atypical/commit/standard"`}
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
        <SectionHeading>Bring a grammar, or start from one.</SectionHeading>
        <Card className="col-span-full p-6">
          <Switcher
            className="flex flex-col gap-6"
            options={presets.map(({ value, label, headers }) => ({
              value,
              label,
              panel: (
                <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
                  <div className="flex flex-col gap-4">
                    <ServerCodeBlock
                      codeblock={{ title: "atypical.toml" }}
                      code={`extends = "npm:@atypical/commit/${value}"`}
                      lang="toml"
                    />
                    <Link
                      to={`/docs/presets/${value}`}
                      className="w-fit text-sm font-medium underline underline-offset-4"
                    >
                      {label} preset →
                    </Link>
                  </div>
                  <ul className="flex flex-col gap-2 font-mono text-sm">
                    {headers.map(([header, valid]) => (
                      <li
                        key={header}
                        className="flex flex-row items-center gap-2"
                      >
                        {valid ? (
                          <RiCheckLine
                            className="size-4 shrink-0 text-fd-primary"
                            aria-label="valid"
                          />
                        ) : (
                          <RiCloseLine
                            className="size-4 shrink-0 text-red-400"
                            aria-label="invalid"
                          />
                        )}
                        {header}
                      </li>
                    ))}
                  </ul>
                </div>
              ),
            }))}
          />
        </Card>
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
            code="npx commit-lint --from origin/main"
            lang="bash"
          />
        </Feature>
        <SectionHeading>Lint the history you already have.</SectionHeading>
        <Feature
          className="col-span-full"
          icon={<RiTerminalLine />}
          title="No hook needed yet."
          description={
            <>
              Every commit since <code>origin/main</code> is checked against the preset.
            </>
          }
        >
          <ServerCodeBlock
            code={`npm i -D @atypical/commit
echo 'extends = "npm:@atypical/commit/conventional"' > atypical.toml
npx commit-lint --from origin/main`}
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

const margin = "text-fd-muted-foreground";
const faint = "text-fd-muted-foreground/60";
const label = "text-fuchsia-500";

/** `commit-lint`'s report for `feat: add a parser`, in ariadne's colors. */
const report = (
  <>
    <span className="text-red-500">[03] Error:</span> Failed to parse commit message
    {"\n   "}
    <span className={margin}>╭─[</span> -:1:1 <span className={margin}>]</span>
    {"\n   "}
    <span className={margin}>│</span>
    {"\n "}
    <span className={margin}>1 │</span> <span className={label}>feat</span>
    <span className={faint}>: add a parser</span>
    {"\n "}
    <span className={faint}>{"  │"}</span> <span className={label}>──┬─</span>
    {"\n "}
    <span className={faint}>{"  │"}</span>
    {"   "}
    <span className={label}>╰───</span> `feat` is not in `keywords`, expected one of: release, undo, add, fix, ref, rem
    {"\n"}
    <span className={margin}>───╯</span>
  </>
);

function TryItOut() {
  return (
    <Card className="col-span-full mx-auto w-full max-w-200 p-2">
      <div className="flex flex-row gap-2">
        <h2 className="text-fd-primary content-center font-mono font-bold uppercase [text-box-trim:both] [text-box-edge:cap_alphabetic] border-2 border-fd-primary/50 px-2 rounded-xl">
          Try it out
        </h2>
        <ServerCodeBlock
          code="npm i -D @atypical/commit"
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
        <Terminal
          lines={[
            { input: true, text: "echo 'feat: add a parser' | npx commit-lint -" },
            { input: false, text: report },
            { input: true, text: "echo 'add(lib)[int]: add a parser' | npx commit-lint -" },
            { input: true, text: "echo $?" },
            { input: false, text: "0" },
          ]}
        />
      </div>
    </Card>
  );
}

export function getConfig() {
  return {
    render: "static",
  };
}
