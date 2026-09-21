import { Link } from "waku";

import FaultyTerminal from "@/components/faulty-terminal";

export default function Home() {
  return (
    <div className="relative flex-1 flex flex-col items-center justify-center text-center">
      <div className="absolute inset-0 -z-10">
        <FaultyTerminal
          scale={1.5}
          digitSize={1.2}
          timeScale={0.5}
          brightness={0.5}
          scanlineIntensity={0.2}
          glitchAmount={0.1}
          curvature={0.1}
          mouseStrength={0.5}
        />
      </div>
      <h1 className="font-medium text-xl mb-4">Fumadocs on Waku.</h1>
      <Link
        to="/docs"
        className="px-3 py-2 rounded-lg bg-fd-primary text-fd-primary-foreground font-medium text-sm mx-auto"
      >
        Open Docs
      </Link>
    </div>
  );
}

export function getConfig() {
  return {
    render: "static",
  };
}
