import { readFile, writeFile } from "node:fs/promises";

const [, , argument] = process.argv;
const checkOnly = argument === "--check";

const packageJson = JSON.parse(await readFile("package.json", "utf8"));
const expectedVersion = checkOnly ? packageJson.version : argument;

if (!expectedVersion || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(expectedVersion)) {
  throw new Error("Usage: node scripts/sync-version.mjs <version> | --check");
}

const packageLock = JSON.parse(await readFile("package-lock.json", "utf8"));
const tauriConfig = JSON.parse(await readFile("src-tauri/tauri.conf.json", "utf8"));
const cargoToml = await readFile("src-tauri/Cargo.toml", "utf8");
const cargoVersion = cargoToml.match(/^version = "([^"]+)"/m)?.[1];

const versions = {
  "package.json": packageJson.version,
  "package-lock.json": packageLock.version,
  "package-lock.json root package": packageLock.packages?.[""]?.version,
  "src-tauri/Cargo.toml": cargoVersion,
  "src-tauri/tauri.conf.json": tauriConfig.version,
};

if (checkOnly) {
  const mismatches = Object.entries(versions).filter(([, version]) => version !== expectedVersion);
  if (mismatches.length > 0) {
    const details = mismatches
      .map(([file, version]) => `${file}: ${version ?? "missing"}`)
      .join("\n");
    throw new Error(`Version mismatch; expected ${expectedVersion}:\n${details}`);
  }
  console.log(`All manifests use version ${expectedVersion}`);
  process.exit(0);
}

packageJson.version = expectedVersion;
packageLock.version = expectedVersion;
if (packageLock.packages?.[""]) packageLock.packages[""].version = expectedVersion;
tauriConfig.version = expectedVersion;
const updatedCargoToml = cargoToml.replace(/^version = "[^"]+"/m, `version = "${expectedVersion}"`);

await Promise.all([
  writeFile("package.json", `${JSON.stringify(packageJson, null, 2)}\n`),
  writeFile("package-lock.json", `${JSON.stringify(packageLock, null, 2)}\n`),
  writeFile("src-tauri/tauri.conf.json", `${JSON.stringify(tauriConfig, null, 2)}\n`),
  writeFile("src-tauri/Cargo.toml", updatedCargoToml),
]);

console.log(`Synced all manifests to version ${expectedVersion}`);
