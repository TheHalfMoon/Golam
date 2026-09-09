import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const args = process.argv.slice(2);
let root = process.cwd();
for (let index = 0; index < args.length; index += 1) {
  if (args[index] === '--root') {
    const value = args[index + 1];
    if (!value) {
      throw new Error('--root requires a value');
    }
    root = path.resolve(value);
    index += 1;
    continue;
  }
  throw new Error(`unsupported argument: ${args[index]}`);
}

const nodeModules = path.join(root, 'node_modules');
if (!fs.existsSync(nodeModules)) {
  throw new Error(`node_modules does not exist: ${nodeModules}`);
}

const deniedPackageNames = new Set(['fsevents']);
const deniedNativeBasenames = new Set(['fsevents.node']);

function listDeniedPaths(directory) {
  const denied = [];
  const stack = [directory];
  while (stack.length > 0) {
    const current = stack.pop();
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const absolute = path.join(current, entry.name);
      if (entry.isDirectory()) {
        if (deniedPackageNames.has(entry.name)) {
          denied.push(absolute);
          continue;
        }
        stack.push(absolute);
        continue;
      }
      if (entry.isFile() && deniedNativeBasenames.has(entry.name)) {
        denied.push(absolute);
      }
    }
  }
  return denied.sort();
}

function removeDeniedPackages(directory) {
  const candidates = [];
  const stack = [directory];
  while (stack.length > 0) {
    const current = stack.pop();
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      if (!entry.isDirectory()) {
        continue;
      }
      const absolute = path.join(current, entry.name);
      if (deniedPackageNames.has(entry.name)) {
        candidates.push(absolute);
        continue;
      }
      stack.push(absolute);
    }
  }

  for (const candidate of candidates.sort()) {
    fs.rmSync(candidate, { recursive: true, force: true, maxRetries: 0 });
  }
}

removeDeniedPackages(nodeModules);

const remaining = listDeniedPaths(nodeModules);
if (remaining.length > 0) {
  throw new Error(`denied native optional remains after prune: ${remaining.join(', ')}`);
}

console.log('DENIED_NATIVE_OPTIONAL_PRUNE=PASS');
console.log(`PRUNE_ROOT=${root}`);
console.log('DENIED_PACKAGE=fsevents');
console.log('DENIED_NATIVE_BASENAME=fsevents.node');
