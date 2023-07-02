import { Attrs } from "./Input.ts";

export class Code {
  indentation = "  " + d`$$`;
  constructor(public readonly lines: (Code | string)[] = []) {}
  static docString(docs: Attrs): string[] {
    let found = "";
    if (docs.rust_docs) found += docs.rust_docs;
    if (docs.serde_attrs || docs.serde_flags) {
      found +=
        "\n\n`#[serde(" +
        Object.keys(docs.serde_flags ?? {})
          .concat(
            Object.entries(docs.serde_attrs ?? {}).map(
              (a) => `${a[0]} = ${JSON.stringify(a[1][0])}`
            )
          )
          .join(", ") +
        ")]`";
    }
    if (docs.codegen_attrs || docs.codegen_flags) {
      found +=
        "\n\n`#[codegen(" +
        Object.keys(docs.codegen_flags ?? {})
          .concat(
            Object.entries(docs.codegen_attrs ?? {}).map(
              (a) => `${a[0]} = ${JSON.stringify(a[1][0])}`
            )
          )
          .join(", ") +
        ")]`";
    }

    found = found.trim();

    if (found) {
      return [
        found.includes("\n")
          ? "/**\n * " +
            found
              .trim()
              .replace(/\n([^\n])/g, "\n $1")
              .replace(/\n/g, "\n *") +
            "\n */"
          : "/** " + found.trim() + " */",
      ];
    } else {
      return [];
    }
  }
  add(arr: TemplateStringsArray, ...args: Args) {
    this.lines.push(raw(arr, ...args));
  }
  addDocString(docs: Attrs | undefined | null) {
    if (!docs) return;
    this.lines.push(...Code.docString(docs));
  }
  ad1(arr: TemplateStringsArray, ...args: Args) {
    const last = this.lines.findLast(() => true);
    if (last instanceof Code) {
      last.add(arr, ...args);
    } else {
      this.lines.push(new Code([raw(arr, ...args)]));
    }
  }
  indented(): Code {
    const last = this.lines.findLast(() => true);
    if (last instanceof Code) return last;
    const c = new Code();
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
        const toAdd = line.toStringIndented(indentation, level + 1);
        if (toAdd.length) {
          if (str.length) str += delimiter + indentation + d`5`;
          str += toAdd;
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
    })
  );
}
const DEBUG = false;
function d(tmpl: TemplateStringsArray, ...subs: Args): string {
  if (DEBUG) return `/*${raw(tmpl, ...subs)}*/ `;
  return "";
}
