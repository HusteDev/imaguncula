// SPDX-License-Identifier: MIT OR Apache-2.0
import { spawn, spawnSync } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const executable = process.platform === "win32" ? "imaguncula-desktop.exe" : "imaguncula-desktop";
const application = path.join(root, "target", "debug", executable);
const driverExecutable = process.platform === "win32" ? "tauri-driver.exe" : "tauri-driver";
const driver =
  process.env.IMAGUNCULA_TAURI_DRIVER ?? path.join(os.homedir(), ".cargo", "bin", driverExecutable);
let driverProcess;
let shuttingDown = false;

export const config = {
  runner: "local",
  host: "127.0.0.1",
  port: 4444,
  path: "/",
  specs: ["./specs/**/*.e2e.mjs"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": { application },
    },
  ],
  logLevel: "warn",
  bail: 0,
  waitforTimeout: 10_000,
  connectionRetryTimeout: 120_000,
  connectionRetryCount: 1,
  services: [],
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    ui: "bdd",
    timeout: 60_000,
  },
  onPrepare() {
    const pnpm = process.platform === "win32" ? "pnpm.cmd" : "pnpm";
    const result = spawnSync(pnpm, ["tauri", "build", "--debug", "--no-bundle"], {
      cwd: root,
      stdio: "inherit",
    });
    if (result.error) throw result.error;
    if (result.status !== 0) throw new Error(`Tauri build failed with status ${result.status}`);
  },
  beforeSession() {
    driverProcess = spawn(driver, [], { stdio: ["ignore", "inherit", "inherit"] });
    driverProcess.once("error", failDriver);
    driverProcess.once("exit", (code) => {
      if (!shuttingDown) failDriver(new Error(`tauri-driver exited with status ${code}`));
    });
  },
  afterSession() {
    stopDriver();
  },
  onComplete() {
    stopDriver();
  },
};

function failDriver(error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
}

function stopDriver() {
  shuttingDown = true;
  driverProcess?.kill();
  driverProcess = undefined;
}

const shutdownSignals =
  process.platform === "win32"
    ? ["SIGINT", "SIGTERM", "SIGBREAK"]
    : ["SIGINT", "SIGTERM", "SIGHUP"];
for (const signal of shutdownSignals) {
  process.once(signal, () => {
    stopDriver();
    process.kill(process.pid, signal);
  });
}
