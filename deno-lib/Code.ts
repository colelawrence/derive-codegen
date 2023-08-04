import { gen } from "./gen.ts";
export class Code {
  indentation = "  " + d`$$`;
  constructor(
    public readonly lines: (Code | string)[] = [],
    public readonly isGroup = false,
  ) {}
  static docStringSettings = {
    multi_line: { prefix: "/**", empty_line_pre: "\n *", line_pre: "\n * ", suffix: "\n */" },
    single_line: { prefix: "/** ", suffix: " */" },
  };
  static docString(docs: gen.Attrs, extraLine?: string, includeLocationID?: [string, gen.LocationID]): string[] {
    let found = "";
    if (docs.rust_docs) found += docs.rust_docs;
    if (docs.serde_attrs || docs.serde_flags) {
      found +=
        "\n\n`#[serde(" +
        Object.keys(docs.serde_flags ?? {})
          .concat(Object.entries(docs.serde_attrs ?? {}).map((a) => `${a[0]} = ${JSON.stringify(a[1][0])}`))
          .join(", ") +
        ")]`";
    }
    if (docs.codegen_attrs || docs.codegen_flags) {
      found +=
        "\n\n`#[codegen(" +
        Object.keys(docs.codegen_flags ?? {})
          .concat(Object.entries(docs.codegen_attrs ?? {}).map((a) => `${a[0]} = ${JSON.stringify(a[1][0])}`))
          .join(", ") +
        ")]`";
    }
    if (includeLocationID) {
      // Future: You could also maybe create a configurable declaration map so jumping to definition
      // can go to the actual Rust source file.
      const [prefix, loc] = includeLocationID;
      // Sample: `L(hn-design-tools/src/color.rs:16 #B6019-B6033)`
      const link = loc
        .replace(/^L\(((?:[^\/]+\/)*)([^:]+)(\S*)\s*#B\d+-B\d+\)$/, `[Source \`$1$2$3\`](__prefix__$1$2)`)
        .replace("__prefix__", prefix);
      found += `\n\n${link}`;
    }

    if (extraLine) {
      found += `\n\n${extraLine}`;
    }

    found = found.trim();

    if (found) {
      return [
        found.includes("\n")
          ? Code.docStringSettings.multi_line.prefix +
            Code.docStringSettings.multi_line.line_pre +
            found.trim().replace(/\n([^\n]*)/g, (_, capture) => {
              if (capture.length) return Code.docStringSettings.multi_line.line_pre + capture;
              return Code.docStringSettings.multi_line.empty_line_pre + capture;
            }) +
            Code.docStringSettings.multi_line.suffix
          : Code.docStringSettings.single_line.prefix + found.trim() + Code.docStringSettings.single_line.suffix,
      ];
    } else {
      return [];
    }
  }
  get lastLine(): string {
    const last = this.lines[this.lines.length - 1];
    if (typeof last !== "string") throw new Error("Expected last line to be a string");
    return last;
  }
  set lastLine(value: string) {
    this.lines[this.lines.length - 1] = value;
  }
  add(arr: TemplateStringsArray, ...args: Args) {
    this.lines.push(raw(arr, ...args));
  }
  addDocString(docs: gen.Attrs | undefined | null, extraLine?: string) {
    if (!docs) return;
    this.lines.push(...Code.docString(docs, extraLine));
  }
  ad1(arr: TemplateStringsArray, ...args: Args) {
    const last = this.lines.findLast(() => true);
    if (last instanceof Code) {
      last.add(arr, ...args);
    } else {
      this.lines.push(new Code([raw(arr, ...args)]));
    }
  }
  grouped(lines?: (Code | string)[]): Code {
    // const last = this.lines.findLast(() => true);
    // if (last instanceof Code && last.group) {
    //   if (lines) last.lines.push(...lines);
    //   return last;
    // }
    const c = new Code(lines, true);
    this.lines.push(c);
    return c;
  }
  indented(lines?: (Code | string)[]): Code {
    const last = this.lines.findLast(() => true);
    if (last instanceof Code && !last.isGroup) {
      if (lines) last.lines.push(...lines);
      return last;
    }
    const c = new Code(lines);
    this.lines.push(c);
    return c;
  }
  scope(fn: (c: Code) => void) {
    const c = new Code();
    this.lines.push(c);
    fn(c);
  }
  toStringIndented(indentation: string, level: number): string {
    const indent = indentation.repeat(level);
    const delimiter = "\n" + d`L${level}` + indent;
    let str = "";
    for (const line of this.lines) {
      if (typeof line === "string") {
        if (str.length) str += delimiter + d`4`;
        str += line.replace(/\n([^\n])/g, delimiter + "$1");
      } else {
        if (line.isGroup) {
          const toAdd = line.toStringIndented(indentation, level);
          if (toAdd.length) {
            if (str.length) str += delimiter + d`3`;
            str += toAdd;
          }
        } else {
          const toAdd = line.toStringIndented(indentation, level + 1);
          if (toAdd.length) {
            if (str.length) str += delimiter + indentation + d`5`;
            str += toAdd;
          }
        }
      }
    }
    return str;
  }

  toString() {
    // console.error(Deno.inspect(this, { colors: true, depth: Infinity }));
    return this.toStringIndented(this.indentation, 0);
  }
}

type Args = (string | number | null | false | undefined | { src: string })[];
function raw(template: TemplateStringsArray, ...args: Args) {
  return String.raw(
    template,
    ...args.map((a) => {
      if (a == null || a === false) return "";
      return typeof a === "object" && "src" in a ? a.src : a;
    }),
  );
}
const DEBUG = false;
function d(tmpl: TemplateStringsArray, ...subs: Args): string {
  if (DEBUG) return `/*${raw(tmpl, ...subs)}*/ `;
  return "";
}
