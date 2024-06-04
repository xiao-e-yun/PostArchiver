import fs from "fs";
import path from "path";
import { execSync } from "child_process";

const directory = "bindings";
const files = fs.readdirSync(directory);

let tags = "v0.0.0";
try {
  tags = execSync("git describe --tags | grep -oP '^v?\K[0-9]+\.[0-9]+\.[0-9]+'").toString().trim();
} catch {}

let indexTs = "// This file is generated automatically\n";
indexTs += `\n// Build Tags: v${tags}\n`;

for (const file of files) {
  if (file === "index.ts") continue;
  indexTs += `export * from './${file}';\n`;
}

fs.writeFileSync(path.join(directory, "index.ts"), indexTs);

const packageJson = `{
  "name": "post-archiver",
  "description": "Types for Post Archiver, https://github.com/xiao-e-yun/PostArchiver",
  "version": "${tags}",
  "types": "./index.ts",
  "repository": {
    "type": "git",
    "url": "git+https://github.com/xiao-e-yun/PostArchiver.git"
  },
  "author": "xiao-e-yun",
  "license": "BSD-3-Clause",
  "homepage": "https://github.com/xiao-e-yun/PostArchiver"
}`;

fs.writeFileSync(path.join(directory, "package.json"), packageJson);

const readme = fs.readFileSync("README.md", "utf-8")
fs.writeFileSync(path.join(directory, "README.md"), readme);