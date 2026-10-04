import vm from "node:vm";
import { readFile } from "node:fs/promises";
import ts from "../node_modules/typescript/lib/typescript.js";
export async function localeModule(context) {
 const root=new URL("../src/i18n.ts",import.meta.url);
 const module=new vm.SourceTextModule(ts.transpile(await readFile(root,"utf8"),{module:ts.ModuleKind.ESNext}),{context});
 await module.link(async specifier=>{
  if(specifier==="react")return new vm.SyntheticModule(["useSyncExternalStore"],function(){this.setExport("useSyncExternalStore",(_subscribe,get)=>get());},{context});
  const value=JSON.parse(await readFile(new URL(specifier,root),"utf8"));
  return new vm.SyntheticModule(["default"],function(){this.setExport("default",value);},{context});
 });
 return module;
}
