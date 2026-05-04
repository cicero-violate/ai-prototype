#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import process from "node:process";

const REQUIRED = "required";
const ADVISORY = "advisory";
const OPTIONAL = "optional";

const STEPS = {
  check: { command: ["npm", "run", "check"], mode: REQUIRED },
  test: { command: ["npm", "test"], mode: REQUIRED },
  smoke: { command: ["npm", "run", "smoke"], mode: REQUIRED },
  legacy: { command: ["npm", "run", "test:artifacts:legacy"], mode: ADVISORY },
  live: { command: ["npm", "run", "test:live"], mode: OPTIONAL },
};

const PROFILES = {
  offline: ["check", "test", "smoke"],
  legacy: ["legacy"],
  live: ["live"],
  release: ["check", "test", "smoke", "legacy", "live"],
};

const ENVIRONMENT_FAILURE = /ECONNREFUSED|cdp_unavailable|healthz status=502|router health check|127\.0\.0\.1:9221/;

function parseArgs(argv) {
  return {
    profile: argv.find((arg) => !arg.startsWith("--")) ?? "release",
    details: argv.includes("--details"),
    forceLive: argv.includes("--force-live") || argv.includes("--force"),
  };
}

function liveEnabled({ profile, forceLive, env }) {
  return forceLive || profile === "live" || env.RUN_LIVE_TESTS === "1" || Boolean(env.LIVE_ROUTER_URL);
}

function commandText(command) {
  return command.join(" ");
}

function tail(text, limit = 16) {
  return text.split(/\r?\n/).filter(Boolean).slice(-limit);
}

export function classifyStep({ status, text }) {
  if (status === 0) return "pass";
  return ENVIRONMENT_FAILURE.test(text) ? "fail_environment" : "fail_source";
}

function runStep(name, options) {
  const step = STEPS[name];
  if (name === "live" && !liveEnabled(options)) {
    return { name, mode: step.mode, command: commandText(step.command), status: "skipped_environment" };
  }
  if (name === "live") return runLiveStep(step, options);
  return runCommandStep(name, step, options);
}

function runLiveStep(step, options) {
  const host = process.env.LIVE_CDP_HOST ?? "127.0.0.1";
  const port = process.env.LIVE_CDP_PORT ?? "9221";
  const url = `http://${host}:${port}/json/version`;
  const probe = spawnSync(process.execPath, ["-e", `
    const ac = new AbortController();
    setTimeout(() => ac.abort(), 2500).unref();
    fetch(${JSON.stringify(url)}, { signal: ac.signal })
      .then((r) => process.exit(r.ok ? 0 : 1))
      .catch((e) => { console.error(e.message); process.exit(1); });
  `], { encoding: "utf8", timeout: 5_000 });

  if (probe.status !== 0) {
    return {
      name: "live",
      mode: step.mode,
      command: `GET ${url}`,
      status: "fail_environment",
      exit_code: probe.status ?? 1,
      output_tail: options.details ? tail(`${probe.stdout ?? ""}${probe.stderr ?? ""}${probe.error?.message ?? ""}`) : undefined,
    };
  }

  return runCommandStep("live", step, options);
}

function runCommandStep(name, step, options) {
  const startedAt = new Date().toISOString();
  const result = spawnSync(step.command[0], step.command.slice(1), {
    cwd: process.cwd(),
    encoding: "utf8",
    env: process.env,
    maxBuffer: 10_000_000,
  });
  const text = `${result.stdout ?? ""}${result.stderr ?? ""}${result.error?.message ?? ""}`;
  const status = classifyStep({ status: result.status ?? 1, text });
  return {
    name,
    mode: step.mode,
    command: commandText(step.command),
    status,
    exit_code: result.status ?? 1,
    started_at: startedAt,
    finished_at: new Date().toISOString(),
    output_tail: status === "pass" && !options.details ? undefined : tail(text),
  };
}

export function runValidationProfile(profile, options = {}) {
  if (!PROFILES[profile]) throw new Error(`unknown validation profile: ${profile}`);
  const opts = { env: process.env, profile, details: false, forceLive: false, ...options };
  const steps = PROFILES[profile].map((name) => runStep(name, opts));
  const requiredFailed = steps.some((s) => s.mode === REQUIRED && s.status !== "pass");
  const liveSourceFailed = profile === "live" && steps.some((s) => s.status === "fail_source");
  const liveEnvironmentFailed = profile === "live" && steps.some((s) => s.status === "fail_environment");
  return {
    schema: "ai_chromium.validation_profile.v1",
    profile,
    status: requiredFailed || liveSourceFailed ? "fail" : liveEnvironmentFailed ? "fail_environment" : "pass",
    summary: {
      required_pass: !requiredFailed,
      advisory_failures: steps.filter((s) => s.mode === ADVISORY && s.status !== "pass").length,
      optional_environment_blocks: steps.filter((s) => s.mode === OPTIONAL && s.status === "fail_environment").length,
      optional_skips: steps.filter((s) => s.status === "skipped_environment").length,
    },
    steps,
  };
}

function main() {
  try {
    const args = parseArgs(process.argv.slice(2));
    const report = runValidationProfile(args.profile, args);
    console.log(JSON.stringify(report, null, 2));
    if (report.status !== "pass") process.exitCode = 1;
  } catch (error) {
    console.error(error.message);
    process.exitCode = 2;
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main();