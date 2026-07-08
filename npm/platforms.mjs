// The platform packages: npm name suffix -> cargo target and os/cpu
// manifest fields.
export default {
  "linux-x64": { target: "x86_64-unknown-linux-gnu", os: "linux", cpu: "x64" },
  "linux-arm64": { target: "aarch64-unknown-linux-gnu", os: "linux", cpu: "arm64" },
  "darwin-x64": { target: "x86_64-apple-darwin", os: "darwin", cpu: "x64" },
  "darwin-arm64": { target: "aarch64-apple-darwin", os: "darwin", cpu: "arm64" },
  "win32-x64": { target: "x86_64-pc-windows-msvc", os: "win32", cpu: "x64" },
  "win32-arm64": { target: "aarch64-pc-windows-msvc", os: "win32", cpu: "arm64" },
};
