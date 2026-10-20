"use client";

import { type CSSProperties, type ReactNode, useEffect, useRef, useState } from "react";
import { Link } from "waku";

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { Bench } from "@/lib/bench";
import { cn } from "@/lib/utils";

type Metric = "latency" | "size" | "packages";

interface Bar {
  name: string;
  value: string;
  segments: { amount: number; className: string }[];
}

interface Chart {
  title: string;
  unit: string;
  tick: (amount: number) => string;
  bars: Bar[];
}

const metrics: { value: Metric; label: string }[] = [
  { value: "latency", label: "Latency" },
  { value: "size", label: "Install size" },
  { value: "packages", label: "Packages" },
];

const ours = "bg-fd-primary";
const theirs = "bg-fd-muted-foreground/60";
const runtime = "bg-fd-muted-foreground/25";

const mb = (bytes: number) => bytes / 1e6;

function charts({ latency, size, packages }: Omit<Bench, "lanes" | "file">): Record<Metric, Chart> {
  return {
    latency: {
      title: "Linting a valid header",
      unit: "mean wall time · milliseconds (lower is better)",
      tick: (ms) => `${ms}ms`,
      bars: [
        {
          name: "atypical",
          value: `${latency.atypical}ms`,
          segments: [{ amount: latency.atypical, className: ours }],
        },
        {
          name: "commitlint",
          value: `${latency.commitlint}ms`,
          segments: [{ amount: latency.commitlint, className: theirs }],
        },
      ],
    },
    size: {
      title: "Install footprint",
      unit: "megabytes on disk (lower is better) · lighter: the Node.js runtime",
      tick: (amount) => `${amount}MB`,
      bars: [
        {
          name: "atypical",
          value: `${mb(size.atypical).toFixed(1)}MB`,
          segments: [{ amount: mb(size.atypical), className: ours }],
        },
        {
          name: "commitlint",
          value: `${mb(size.commitlint + size.node).toFixed(0)}MB`,
          segments: [
            { amount: mb(size.commitlint), className: theirs },
            { amount: mb(size.node), className: runtime },
          ],
        },
      ],
    },
    packages: {
      title: "Packages installed",
      unit: "count (lower is better)",
      tick: (amount) => `${amount}`,
      bars: [
        {
          name: "atypical",
          value: `${packages.atypical}`,
          segments: [{ amount: packages.atypical, className: ours }],
        },
        {
          name: "commitlint",
          value: `${packages.commitlint}`,
          segments: [{ amount: packages.commitlint, className: theirs }],
        },
      ],
    },
  };
}

/** A 1-2-5 step that splits `max` into at most five intervals. */
function axis(max: number) {
  const magnitude = 10 ** Math.floor(Math.log10(max / 5));
  const step = [1, 2, 5, 10].map((m) => m * magnitude).find((s) => max / s <= 5) ?? magnitude * 10;
  const end = Math.ceil(max / step) * step;

  return { end, ticks: Array.from({ length: end / step + 1 }, (_, i) => i * step) };
}

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

  const chart = charts(bench)[metric];
  const totals = chart.bars.map((bar) => bar.segments.reduce((sum, { amount }) => sum + amount, 0));
  const { end, ticks } = axis(Math.max(...totals));

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

      <div className="flex flex-row items-start justify-between gap-4">
        <div>
          <h3 className="font-medium">{chart.title}</h3>
          <p className="text-sm text-fd-muted-foreground">{chart.unit}</p>
        </div>
        {metric === "latency" && (
          <button
            type="button"
            className="shrink-0 text-sm text-fd-muted-foreground hover:text-fd-foreground"
            onClick={() => setRun((n) => n + 1)}
          >
            <span className="text-fd-primary">▶</span> replay at 1/1000 speed
          </button>
        )}
      </div>

      <div className="flex flex-col gap-3">
        {chart.bars.map((bar, index) => (
          <Row
            key={bar.name}
            name={bar.name}
            value={bar.value}
            highlight={index === 0}
          >
            {bar.segments.map((segment) =>
              metric === "latency" ? (
                <Race
                  key={`${segment.className}-${run}`}
                  run={run}
                  share={segment.amount / end}
                  seconds={segment.amount}
                  className={segment.className}
                />
              ) : (
                <div
                  key={segment.className}
                  className={cn("h-full min-w-1", segment.className)}
                  style={{ width: `${(segment.amount / end) * 100}%` }}
                />
              ),
            )}
          </Row>
        ))}
        <Row>
          <div className="relative h-5 w-full border-t border-fd-border text-[0.65rem] text-fd-muted-foreground font-mono">
            {ticks.map((tick) => (
              <span
                key={tick}
                className="absolute top-1 -translate-x-1/2"
                style={{ left: `${(tick / end) * 100}%` }}
              >
                {chart.tick(tick)}
              </span>
            ))}
          </div>
        </Row>
      </div>

      <div className="flex flex-row items-start justify-between gap-4 border-t pt-4 text-xs text-fd-muted-foreground">
        <p>
          hyperfine, 100 runs after 10 warmup · {bench.cpu} · {bench.date}
        </p>
        <Link
          to="/docs/benchmarks"
          className="shrink-0 underline underline-offset-2"
        >
          reproduce
        </Link>
      </div>
    </div>
  );
}

function Row({
  name,
  value,
  highlight,
  children,
}: {
  name?: string;
  value?: string;
  highlight?: boolean;
  children: ReactNode;
}) {
  return (
    <div className="grid grid-cols-[5.5rem_1fr_4.5rem] items-center gap-3 text-sm">
      <span className={cn(highlight && "font-medium")}>{name}</span>
      <div className={cn("flex", name && "h-6 overflow-hidden rounded-sm bg-fd-muted")}>{children}</div>
      <span
        className={cn("text-end font-mono tabular-nums", highlight ? "font-bold text-base" : "text-fd-muted-foreground")}
      >
        {value}
      </span>
    </div>
  );
}

/**
 * Grows to `share` of the axis over `seconds`, the measured milliseconds slowed down 1000×, so every bar grows at the
 * same speed and stops at its own value. Restarts whenever `run` changes.
 */
function Race({ run, share, seconds, className }: { run: number; share: number; seconds: number; className: string }) {
  return (
    <div
      className={cn("h-full w-0 min-w-1 motion-reduce:w-(--to) motion-reduce:animate-none!", className)}
      style={
        {
          "--to": `${share * 100}%`,
          animation: run > 0 ? `bench-race ${seconds}s linear forwards` : undefined,
        } as CSSProperties
      }
    />
  );
}
