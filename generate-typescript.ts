import { Code } from "./Code.ts";
import { Input, Output, NamedField, Format, Attrs } from "./Input.ts";
import { match } from "./match.ts";

function convert(input: Input): Output {
  const generated = new Code();
  console.error("Number of declarations: ", input.declarations.length);
  for (const decl of input.declarations) {
    const $decl = new Code();
    const docs = Code.docString(decl);
    match(decl.container_kind)
      .Struct(({ fields }) => {
        const structIdent = ident(decl.id);
        // type
        $decl.lines.push(...docs);
        $decl.add`export interface ${structIdent} {`;
        typeFields$($decl.indented(), fields);
        $decl.add`}`;
        // create
        $decl.lines.push(...docs);
        $decl.add`export function ${structIdent}(inner: ${structIdent}): ${structIdent} {`;
        $decl.ad1`return inner;`;
        $decl.add`}`;
      })
      .Enum(({ repr, variants }) => {
        const enumIdent = ident(decl.id);
        const $nsMatchToObj = new Code(["// callbacks"]);
        const $nsMatchIfStrs = new Code(["// if-else strings"]);
        const $nsMatchIfObjs = new Code([
          "// if-else objects",
          `if (typeof input !== "object" || input == null) throw new TypeError("Unexpected non-object for input");`,
        ]);
        const $ns = new Code([
          `/** Match helper for {@link ${enumIdent}} */`,
          `export function match<R>(`,
          new Code([`to: {`, $nsMatchToObj, `},`]),
          `): (input: ${enumIdent}) => R {`,
          new Code([
            `return function _match(input): R {`,
            $nsMatchIfStrs,
            $nsMatchIfObjs,
            new Code([`const _exhaust: never = input;`, `return _exhaust;`]),
            `}`,
          ]),
          `}`,
        ]);
        const typeCode = new Code([
          ...docs,
          // create / matchers
          `// deno-lint-ignore no-namespace`,
          `export namespace ${enumIdent} {`,
          $ns,
          `}`,
          // type
          ...docs,
          `export type ${enumIdent} =`,
        ]);

        // TODO: handle different representations properly

        for (const variant of variants) {
          const variantIdent = ident(variant.id);
          const variantNameField = namedField(variant);
          const variantIdentRef = `${enumIdent}.${variantIdent}`;
          typeCode.ad1`| ${variantIdentRef}`;
          const variantDocs = Code.docString(variant);
          match(variant.variant_format)
            .NewType((format) => {
              const newTypeTs = createFormat(format);
              // type
              $ns.lines.push(...variantDocs);
              $ns.add`export type ${variantIdent} = {`;
              $ns.indented().lines.push(...variantDocs);
              $ns.ad1`${variantNameField}: ${newTypeTs.src}`;
              $ns.add`};`;
              // create
              $ns.lines.push(...variantDocs);
              $ns.add`export function ${variantIdent}(value${
                newTypeTs.optional && "?"
              }: ${newTypeTs.src}): ${variantIdent} {`;
              $ns.ad1`return { ${variantNameField}: value };`;
              $ns.add`}`;
              // match callback
              $nsMatchToObj.lines.push(...variantDocs);
              $nsMatchToObj.add`${variantIdent}(inner: ${variantIdent}[${namedStr(
                variant
              )}]): R;`;
              // match if else
              $nsMatchIfObjs.add`if (${namedStr(
                variant
              )} in input) return to.${variantNameField}(input[${namedStr(
                variant
              )}]);`;
            })
            .Tuple((formats) => {
              const formatTsList = tupleFormats(formats);
              const vnStr = namedStr(variant);
              const innerTypeRef = `[${formatTsList
                .map((f) => f.fmt.src)
                .join(", ")}]`;
              // type
              $ns.lines.push(...variantDocs);
              $ns.add`export type ${variantIdent} = { ${variantNameField}: ${innerTypeRef} };`;
              $ns.lines.push(...variantDocs);
              // create
              $ns.add`export function ${variantIdent}(${formatTsList
                .map((f) => `${f.id}: ${f.fmt.src}`)
                .join(", ")}): ${variantIdent} {`;
              $ns.ad1`return { ${variantNameField}: [${formatTsList
                .map((f) => f.id)
                .join(", ")}] };`;
              $ns.add`}`;
              // match
              $nsMatchToObj.lines.push(...variantDocs);
              $nsMatchToObj.add`${variantIdent}(inner: ${innerTypeRef}): R,`;
              $nsMatchIfObjs.add`if (${vnStr} in input) return to.${variantIdent}(input[${vnStr}]);`;
            })
            .Unit(() => {
              // type
              $ns.lines.push(...variantDocs);
              $ns.add`export type ${variantIdent} = ${namedStr(variant)}`;
              // create
              $ns.lines.push(...variantDocs);
              $ns.add`export function ${variantIdent}(): ${variantIdent} {`;
              $ns.ad1`return ${namedStr(variant)};`;
              $ns.add`}`;
              // match
              $nsMatchToObj.lines.push(...variantDocs);
              $nsMatchToObj.add`${variantIdent}(): R,`;
              $nsMatchIfStrs.add`if (input === ${namedStr(
                variant
              )}) return to.${variantIdent}();`;
            })
            .Struct(({ fields }) => {
              const innerTypeRef = `${variantIdent}[${namedStr(variant)}]`;
              // type
              $ns.lines.push(...variantDocs);
              $ns.add`export type ${variantIdent} = {`;
              $ns.scope(($$) => {
                $$.add`${variantNameField}: {`;
                typeFields$($$.indented(), fields);
                $$.add`}`;
              });
              $ns.add`}`;
              // create
              $ns.lines.push(...variantDocs);
              $ns.add`export function ${variantIdent}(value: ${innerTypeRef}): ${variantIdent} {`;
              $ns.ad1`return { ${variantNameField}: value }`;
              $ns.add`}`;
              // match
              $nsMatchToObj.lines.push(...variantDocs);
              $nsMatchToObj.add`${variantIdent}(inner: ${innerTypeRef}): R,`;
              $nsMatchIfObjs.add`if (${namedStr(
                variant
              )} in input) return to.${variantIdent}(input[${namedStr(
                variant
              )}]);`;
            })
            .$();
        }

        $decl.lines.push(...typeCode.lines);
      })
      .NewTypeStruct((format) => {
        const structIdent = ident(decl.id);
        const newTypeFormat = createFormat(format);
        // type
        $decl.lines.push(...docs);
        if (decl.serde_flags?.transparent) {
          $decl.add`export type ${structIdent} = ${newTypeFormat.src}`;
        } else {
          $decl.add`export type ${structIdent} = [${newTypeFormat.src}]`;
        }
        // create
        $decl.lines.push(...docs);
        $decl.add`export function ${structIdent}(inner${
          newTypeFormat.optional && "?"
        }: ${newTypeFormat.src}): ${structIdent} {`;
        if (decl.serde_flags?.transparent) {
          $decl.ad1`return inner;`;
        } else {
          $decl.ad1`return [inner];`;
        }
        $decl.add`}`;
      })
      .TupleStruct((formats) => {
        const formatTsList = tupleFormats(formats);
        const structIdent = ident(decl.id);
        // type
        $decl.lines.push(...docs);
        $decl.add`export type ${structIdent} = [${formatTsList
          .map((f) => f.fmt.src)
          .join(", ")}]`;
        // create
        $decl.lines.push(...docs);
        $decl.add`export function ${structIdent}(${formatTsList
          .map((f) => `${f.id}: ${f.fmt.src}`)
          .join(", ")}): ${structIdent} {`;
        $decl.ad1`return [${formatTsList.map((f) => f.id).join(", ")}];`;
        $decl.add`}`;
      })
      .UnitStruct(() => {
        const structIdent = ident(decl.id);
        // type
        $decl.lines.push(...docs);
        $decl.add`export interface ${structIdent} {} /* hmm unit struct? */`;
        // create
        $decl.lines.push(...docs);
        $decl.add`export function ${structIdent}(): ${structIdent} {`;
        $decl.ad1`return {};`;
        $decl.add`}`;
      })
      .$();
    generated.lines.push(...$decl.lines);
  }
  return {
    errors: [],
    files: [
      {
        path: "types.ts",
        source: generated.toString(),
      },
    ],
    warnings: [],
  };
}

/** Only for the types */
function typeFields$($: Code, fields: NamedField[]) {
  for (const field of fields) {
    const { src, optional } = createFormat(field.format);
    const isOptional =
      optional ||
      (field.serde_flags?.default && field.serde_attrs?.skip_serializing_if);
    $.addDocString(field);
    $.add`${namedField(field)}${isOptional && "?"}: ${src}${
      isOptional && " | null | undefined"
    };`;
  }
}

function tupleFormats(formats: Format[]) {
  const aCp = "a".codePointAt(0)!;
  return formats.map((f, idx) => ({
    fmt: createFormat(f),
    id: String.fromCodePoint(aCp + idx),
  }));
}

function createFormat(format: Format): { src: string; optional?: boolean } {
  const num = () => ({ src: "number" });
  if (!format) throw new Error("format not specified");

  return match(format)
    .TypeName((value) => ({ src: ident(value) }))
    .I8(num)
    .I16(num)
    .I32(num)
    .I64(num)
    .I128(num)
    .ISIZE(num)
    .U8(num)
    .U16(num)
    .U32(num)
    .U64(num)
    .U128(num)
    .USIZE(num)
    .F32(num)
    .F64(num)
    .Bool(() => ({ src: "boolean" }))
    .Bytes(() => ({ src: "/* bytes? */ string" }))
    .Never(() => ({ src: "never" }))
    .Char(() => ({ src: "/* char */ string" }))
    .Map(({ key, value }) => ({
      src: `Record<${createFormat(key).src}, ${createFormat(value).src}>`,
    }))
    .Unit(() => ({ src: "/* unit */ null" }))
    .Option((format) => {
      const inner = createFormat(format);
      if (inner.optional) return inner;
      return {
        src: `${inner.src} | undefined | null`,
        optional: true,
      };
    })
    .Incomplete(({ debug }) => ({ src: `/* Incomplete: ${debug} */ unknown` }))
    .Seq((seq) => ({
      src: `Array<${createFormat(seq).src}>`,
    }))
    .Tuple((tuple) => ({
      src: `[${tuple.map((tup) => createFormat(tup).src).join(", ")}]`,
    }))
    .TupleArray(({ content, size }) => ({
      src: `[<${new Array(size).fill(createFormat(content).src).join(", ")}]`,
    }))
    .Str(() => ({ src: "string" }))
    .$();
}

function ident(id: string): string {
  return id.replace(/[^a-zA-Z0-9\$\_]/g, "$").replace(/^(\d)/, "$$1");
}

function namedField(named: { id: string } & Attrs): string {
  const nam = named.serde_attrs?.["rename"]?.[0] ?? named.id;
  if (/^[\w$][\w\d$]*$/.test(nam)) return nam;
  else return JSON.stringify(nam);
}
function namedStr(named: { id: string } & Attrs): string {
  const nam = named.serde_attrs?.["rename"]?.[0] ?? named.id;
  return JSON.stringify(nam);
}

console.log(JSON.stringify(convert(JSON.parse(Deno.args[0])), null, 2));
