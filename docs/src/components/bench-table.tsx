import { latestBench } from "@/lib/bench";
import { gitConfig } from "@/lib/shared";

const megabytes = (bytes: number) => `${(bytes / 1e6).toFixed(1)} MB`;

export function BenchTable() {
  const bench = latestBench();

  if (!bench) return <p>No benchmark snapshot with the commitlint lanes is committed.</p>;

  const source = `https://github.com/${gitConfig.user}/${gitConfig.repo}/blob/${gitConfig.branch}/benches/${bench.file}`;

  return (
    <>
      <table>
        <thead>
          <tr>
            <th>Command</th>
            <th>Mean wall time [ms]</th>
          </tr>
        </thead>
        <tbody>
          {bench.lanes.map((lane) => (
            <tr key={lane.name}>
              <td>
                <code>{lane.name}</code>
              </td>
              <td>{lane.mean}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <table>
        <thead>
          <tr>
            <th>Footprint</th>
            <th>Size</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>
              <code>commit-lint</code> release binary
            </td>
            <td>{megabytes(bench.size.atypical)}</td>
          </tr>
          <tr>
            <td>
              <code>@commitlint/cli</code> with <code>config-conventional</code>
            </td>
            <td>
              {megabytes(bench.size.commitlint)} in {bench.packages.commitlint} packages
            </td>
          </tr>
          <tr>
            <td>Node.js, required by commitlint</td>
            <td>{megabytes(bench.size.node)}</td>
          </tr>
        </tbody>
      </table>
      <p>
        Measured {bench.date} on {bench.cpu}. Source: <a href={source}>{`benches/${bench.file}`}</a>
      </p>
    </>
  );
}
