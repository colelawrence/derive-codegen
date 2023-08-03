import { parseArgs } from "./parseArgs.ts";
import { Code, gen } from "../deno-lib/mod.ts";
import { generateTypeScript } from "../deno-lib/generators/typescript.ts";

const args = parseArgs({
  fileName: "What to call the file with all the declarations",
  prependText: "Prepend this to the top of the file",
  importScalarsFrom: "Relative file path to import all scalars from",
  includeLocationsRelativeTo:
    "Include links to the original source with this prefix",
});

console.log(JSON.stringify(generateTypeScript(JSON.parse(args.jsonInput!), args), null, 2));
