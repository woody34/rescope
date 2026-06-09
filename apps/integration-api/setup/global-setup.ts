import { spawn, ChildProcess } from "child_process";
import { resolve } from "path";

const EMULATOR_BIN =
  process.env.DESCOPE_EMULATOR_BIN ??
  resolve(__dirname, "../../../target/debug/rescope");
const PORT = parseInt(process.env.DESCOPE_EMULATOR_TEST_PORT ?? "4501", 10);
const HEALTH_URL = `http://localhost:${PORT}/health`;
const TIMEOUT_MS = 15_000;

let emulatorProcess: ChildProcess | null = null;

async function pollHealth(): Promise<void> {
  const deadline = Date.now() + TIMEOUT_MS;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(HEALTH_URL);
      if (res.ok) return;
    } catch {
      // not ready yet
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(`Emulator did not become healthy within ${TIMEOUT_MS}ms`);
}

export async function setup(): Promise<void> {
  const env = {
    ...process.env,
    DESCOPE_EMULATOR_PORT: String(PORT),
    DESCOPE_PROJECT_ID: "emulator-project",
    DESCOPE_MANAGEMENT_KEY: "emulator-key",
    RUST_LOG: "error",
  };

  emulatorProcess = spawn(EMULATOR_BIN, [], { env, stdio: "pipe" });

  emulatorProcess.stderr?.on("data", (d: Buffer) => {
    if (process.env.EMULATOR_DEBUG) process.stderr.write(d);
  });

  emulatorProcess.on("exit", (code) => {
    if (code !== 0 && code !== null) {
      console.error(`Emulator exited with code ${code}`);
    }
  });

  await pollHealth();

  // Expose base URL for tests via process.env
  process.env.EMULATOR_BASE_URL = `http://localhost:${PORT}`;
}

export async function teardown(): Promise<void> {
  if (emulatorProcess) {
    emulatorProcess.kill("SIGTERM");
    emulatorProcess = null;
  }
}
