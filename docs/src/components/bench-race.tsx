"use client";

import { type ReactNode, useEffect, useRef, useState } from "react";
import { Link } from "waku";

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { Bench } from "@/lib/bench";
import { cn } from "@/lib/utils";

type Metric = "latency" | "size" | "packages";

const metrics: { value: Metric; label: string }[] = [
  { value: "latency", label: "Latency" },
  { value: "size", label: "Install size" },
  { value: "packages", label: "Packages" },
];

const megabytes = (bytes: number) => `${(bytes / 1e6).toFixed(1)} MB`;

export function BenchRace({ bench }: { bench: Omit<Bench, "lanes" | "file"> }) {
  const [metric, setMetric] = useState<Metric>("latency");
  const [run, setRun] = useState(0);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const node = ref.current;
    if (!node) return;

    const observer = new IntersectionObserver(([entry]) => {
      if (entry?.isIntersecting) setRun((n) => n + 1);
    });

    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  const { latency, size, packages } = bench;
  const sizeMax = size.commitlint + size.node;

  return (
    <div
      ref={ref}
      className="flex flex-col gap-6"
    >
      <ToggleGroup
        variant="outline"
        value={[metric]}
        onValueChange={(value: string[]) => {
          const next = value[0] as Metric | undefined;
          if (!next) return;
          setMetric(next);
          setRun((n) => n + 1);
        }}
      >
        {metrics.map(({ value, label }) => (
          <ToggleGroupItem
            key={value}
            value={value}
          >
            {label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>

      <div className="flex flex-col gap-3">
        {metric === "latency" && (
          <>
            <Row
              name="atypical"
              value={`${latency.atypical} ms`}
            >
              <Race
                run={run}
                seconds={latency.atypical}
                className="bg-fd-primary"
              />
            </Row>
            <Row
              name="commitlint"
              value={`${latency.commitlint} ms`}
            >
              <Race
                run={run}
                seconds={latency.commitlint}
                className="bg-fd-muted-foreground/60"
              />
            </Row>
          </>
        )}
        {metric === "size" && (
          <>
            <Row
              name="atypical"
              value={megabytes(size.atypical)}
            >
              <Segment
                share={size.atypical / sizeMax}
                className="bg-fd-primary"
              />
            </Row>
            <Row
              name="commitlint"
              value={`${megabytes(size.commitlint)}, plus ${megabytes(size.node)} of Node.js`}
            >
              <Segment
                share={size.commitlint / sizeMax}
                className="bg-fd-muted-foreground/60"
              />
              <Segment
                share={size.node / sizeMax}
                className="bg-fd-muted-foreground/25"
              />
            </Row>
          </>
        )}
        {metric === "packages" && (
          <>
            <Row
              name="atypical"
              value={`${packages.atypical} packages`}
            >
              <Segment
                share={packages.atypical / packages.commitlint}
                className="bg-fd-primary"
              />
            </Row>
            <Row
              name="commitlint"
              value={`${packages.commitlint} packages`}
            >
              <Segment
                share={1}
                className="bg-fd-muted-foreground/60"
              />
            </Row>
          </>
        )}
      </div>

      <p className="text-xs text-fd-muted-foreground">
        {metric === "latency" && "Valid header, slowed down 1000×. "}
        {metric === "size" && "The lighter segment is the Node.js runtime commitlint needs. "}
        Measured on {bench.cpu}, {bench.date}.{" "}
        <Link
          to="/docs/benchmarks"
          className="underline underline-offset-2"
        >
          Benchmarks
        </Link>
      </p>
    </div>
  );
}

function Row({ name, value, children }: { name: string; value: string; children: ReactNode }) {
  return (
    <div className="grid grid-cols-[6rem_1fr] items-center gap-3 text-sm">
      <span className="font-medium">{name}</span>
      <div className="flex h-6 overflow-hidden rounded-md bg-fd-muted">
        {children}
        <span className="sr-only">{value}</span>
      </div>
    </div>
  );
}

/** Fills over `seconds`: the measured milliseconds, slowed down 1000×. Restarts whenever `run` changes. */
function Race({ run, seconds, className }: { run: number; seconds: number; className: string }) {
  return (
    <div
      key={run}
      className={cn("h-full w-0 motion-reduce:w-full motion-reduce:animate-none!", className)}
      style={run > 0 ? { animation: `bench-race ${seconds}s linear forwards` } : undefined}
    />
  );
}

function Segment({ share, className }: { share: number; className: string }) {
  return (
    <div
      className={cn("h-full min-w-1", className)}
      style={{ width: `${share * 100}%` }}
    />
  );
}
