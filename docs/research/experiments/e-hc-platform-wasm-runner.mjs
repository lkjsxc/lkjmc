import fs from "node:fs";

const [modulePath, mode, target] = process.argv.slice(2);
if (mode === "--write-probe") {
  try {
    fs.writeFileSync(target, "unexpected write");
    console.log("write-allowed");
  } catch {
    console.log("write-denied");
  }
  process.exit(0);
}
let request;
try {
  request = JSON.parse(fs.readFileSync(0, "utf8"));
} catch {
  request = null;
}
const module = await WebAssembly.compile(fs.readFileSync(modulePath));
const instance = await WebAssembly.instantiate(module);
const valid = request?.subject === "operator" && request?.operation === "inspect";
const allowed = instance.exports.decide(valid ? 1 : 0) === 1;
console.log(JSON.stringify({ decision: allowed ? "allow" : "deny", imports: WebAssembly.Module.imports(module).length }));
