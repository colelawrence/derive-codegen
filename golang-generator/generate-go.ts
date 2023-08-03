import { Code, gen } from "../deno-lib/mod.ts";

Code.docStringSettings = {
  multi_line: { empty_line_pre: "//", line_pre: "// ", prefix: "", suffix: "" },
  single_line: { prefix: "// ", suffix: "" },
}
type GeneratedDecl = {
  code: Code;
  fileName: string;
  package: string;
};
function convert(input: gen.Input): GeneratedDecl[] {
  const generated: GeneratedDecl[] = [];
  console.error("Number of declarations: ", input.declarations.length);
  for (const decl of input.declarations) {
    const packageName = decl.codegen_attrs?.package?.[0] ?? "generated";
    const $decl = new Code([
      `package ${packageName}`,
      "import (",
      new Code(['"encoding/json"', '"fmt"']),
      ")",
      "",
    ]);
    const docs = Code.docString(decl);

    const $support = $decl.grouped(["// supports"]);
    let supportIdx = 0;
    const createFormatFor = createFormatter({
      addSupportingTuple(prefix, tuple) {
        supportIdx += 1;
        const supportIdent = `${prefix}Tuple${
          supportIdx > 1 ? supportIdx : ""
        }`;
        $support.add`type ${supportIdent} interface{} // TODO ${JSON.stringify(
          tuple
        )}`;
        return supportIdent;
      },
    });

    const goDeclIdent = ident(decl.id);
    const createFormat = createFormatFor(goDeclIdent);

    gen.ContainerFormat.match(decl.container_kind, {
      Struct({ fields }) {
        // type
        $decl.lines.push(...docs);
        $decl.add`const todo = \``;
        $decl.add`export type ${goDeclIdent} = {`;
        typeFieldsFinish$(createFormatFor(goDeclIdent), $decl, fields);
        // create
        $decl.lines.push(...docs);
        $decl.add`export function ${goDeclIdent}(inner: ${goDeclIdent}): ${goDeclIdent} {`;
        $decl.ad1`return inner;`;
        $decl.add`}`;
        $decl.add`\``;
      },
      Enum({ repr, variants }) {
        const enumInterfaceIdent = goDeclIdent + "Type";
        $decl.lines.push(...docs);
        $decl.add`type ${goDeclIdent} = ${enumInterfaceIdent}`;
        $decl.add``;
        $decl.add`type ${enumInterfaceIdent} interface{ is${enumInterfaceIdent}() }`;
        $decl.add``;
        const $types = $decl.grouped(["// types"]);
        const $impl = $decl.grouped(["// impl"]);
        const $unmarshl = $decl.grouped(["// unmarshal"]);
        const $marshall = $decl.grouped(["// marshal"]);

        const $nsMatchToObj = new Code(["// callbacks"]);
        const $nsMatchIfStrs = new Code(["// if-else strings"]);
        const $nsMatchIfObjs = new Code([
          "// if-else objects",
          `if (typeof input !== "object" || input == null) throw new TypeError("Unexpected non-object for input");`,
        ]);
        const $ns = new Code([
          `export type ApplyFns<R> = {`,
          $nsMatchToObj,
          `}`,
          `/** Match helper for {@link ${goDeclIdent}} */`,
          `export function apply<R>(`,
          new Code([`to: ApplyFns<R>,`]),
          `): (input: ${goDeclIdent}) => R {`,
          new Code([
            `return function _match(input): R {`,
            $nsMatchIfStrs,
            $nsMatchIfObjs,
            new Code([`const _exhaust: never = input;`, `return _exhaust;`]),
            `}`,
          ]),
          `}`,
          `/** Match helper for {@link ${goDeclIdent}} */`,
          `export function match<R>(`,
          new Code([`input: ${goDeclIdent},`, `to: ApplyFns<R>,`]),
          `): R {`,
          new Code([`return apply(to)(input)`]),
          `}`,
        ]);
        const typeCode = new Code([
          ...docs,
          // create / matchers
          `// deno-lint-ignore no-namespace`,
          `export namespace ${goDeclIdent} {`,
          $ns,
          `}`,
          // type
          ...docs,
          `export type ${goDeclIdent} =`,
        ]);

        // TODO: handle different representations properly

        for (const variant of variants) {
          const variantIdentBare = ident(variant.id);
          const variantGoTypeIdent = `${goDeclIdent}_${variantIdentBare}`;
          $impl.add`func (${variantGoTypeIdent}) is${goDeclIdent}Type() {}`;
          const variantNameField = namedField(variant);
          const variantIdentRef = `${goDeclIdent}.${variantGoTypeIdent}`;
          typeCode.ad1`| ${variantIdentRef}`;
          const variantDocs = Code.docString(variant);
          const createFormat = createFormatFor(variantGoTypeIdent);
          gen.VariantFormat.match(variant.variant_format, {
            NewType(format) {
              const newTypeTs = createFormat(format);
              // type
              $types.lines.push(...variantDocs);
              $types.add`type ${variantGoTypeIdent} ${newTypeTs}`;

              // todo unmarshal
              $unmarshl.add`func(v *${variantGoTypeIdent}) UnmarshalJSON(b []byte) error {`;
              $unmarshl.indented([
                `panic("UnmarshalJSON not implemented for NewType")`,
              ]);
              $unmarshl.add`}`;
              // todo marshal
              $marshall.add`func(v ${variantGoTypeIdent}) MarshalJSON() ([]byte, error) {`;
              $marshall.indented([
                `panic("MarshalJSON not implemented for NewType")`,
              ]);
              $marshall.add`}`;

              // create
              $ns.lines.push(...variantDocs);
              $ns.add`export function ${variantGoTypeIdent}(value${
                newTypeTs.optional && "?"
              }: ${newTypeTs.src}): ${variantGoTypeIdent} {`;
              $ns.ad1`return { ${variantNameField}: value };`;
              $ns.add`}`;
              // match callback
              $nsMatchToObj.lines.push(...variantDocs);
              $nsMatchToObj.add`${variantGoTypeIdent}(inner: ${variantGoTypeIdent}[${namedStr(
                variant
              )}]): R;`;
              // match if else
              $nsMatchIfObjs.add`if (${namedStr(
                variant
              )} in input) return to.${variantNameField}(input[${namedStr(
                variant
              )}]);`;
            },
            Tuple(formats) {
              const formatGoList = tupleFormats(createFormat, formats);
              const vnStr = namedStr(variant);
              const innerTypeRef = `[${formatGoList
                .map((f) => f.fmt.src)
                .join(", ")}]`;
              // type
              $types.lines.push(...variantDocs);
              $types.add`type ${variantGoTypeIdent} struct {`;
              $types.indented([
                "// todo tuple fields like A, B, C",
                ...formatGoList.map((f) => `${f.id} ${f.fmt.src}`),
              ]);
              $types.add`}`;

              // todo unmarshal
              $unmarshl.add`func(v *${variantGoTypeIdent}) UnmarshalJSON(b []byte) error {`;
              $unmarshl.indented([
                `panic("UnmarshalJSON not implemented for NewType")`,
              ]);
              $unmarshl.add`}`;
              // todo marshal
              $marshall.add`func(v ${variantGoTypeIdent}) MarshalJSON() ([]byte, error) {`;
              $marshall.indented([
                `panic("MarshalJSON not implemented for NewType")`,
              ]);
              $marshall.add`}`;
              // typescript type
              $ns.lines.push(...variantDocs);
              $ns.add`export type ${variantGoTypeIdent} = { ${variantNameField}: ${innerTypeRef} };`;
              $ns.lines.push(...variantDocs);
              // create
              $ns.add`export function ${variantGoTypeIdent}(${formatGoList
                .map((f) => `${f.id}: ${f.fmt.src}`)
                .join(", ")}): ${variantGoTypeIdent} {`;
              $ns.ad1`return { ${variantNameField}: [${formatGoList
                .map((f) => f.id)
                .join(", ")}] };`;
              $ns.add`}`;
              // match
              $nsMatchToObj.lines.push(...variantDocs);
              $nsMatchToObj.add`${variantGoTypeIdent}(inner: ${innerTypeRef}): R,`;
              $nsMatchIfObjs.add`if (${vnStr} in input) return to.${variantGoTypeIdent}(input[${vnStr}]);`;
            },
            Unit() {
              // type
              $ns.lines.push(...variantDocs);
              // type
              $types.lines.push(...variantDocs);
              $types.add`type ${variantGoTypeIdent} struct{}`;

              // todo unmarshal
              $unmarshl.add`func(v *${variantGoTypeIdent}) UnmarshalJSON(b []byte) error {`;
              $unmarshl.indented([
                `panic("UnmarshalJSON not implemented for NewType")`,
              ]);
              $unmarshl.add`}`;
              // todo marshal
              $marshall.add`func(v ${variantGoTypeIdent}) MarshalJSON() ([]byte, error) {`;
              $marshall.indented([
                `panic("MarshalJSON not implemented for NewType")`,
              ]);
              $marshall.add`}`;

              $ns.add`export type ${variantGoTypeIdent} = ${namedStr(variant)}`;
              // create
              $ns.lines.push(...variantDocs);
              $ns.add`export function ${variantGoTypeIdent}(): ${variantGoTypeIdent} {`;
              $ns.ad1`return ${namedStr(variant)};`;
              $ns.add`}`;
              // match
              $nsMatchToObj.lines.push(...variantDocs);
              $nsMatchToObj.add`${variantGoTypeIdent}(): R,`;
              $nsMatchIfStrs.add`if (input === ${namedStr(
                variant
              )}) return to.${variantGoTypeIdent}();`;
            },
            Struct({ fields }) {
              const innerTypeRef = `${variantGoTypeIdent}[${namedStr(
                variant
              )}]`;
              // type
              $types.lines.push(...variantDocs);
              $types.add`type ${variantGoTypeIdent} struct {`;
              $types.indented(["// todo shared fields def"]);
              $types.add`}`;

              // todo unmarshal
              $unmarshl.add`func(v *${variantGoTypeIdent}) UnmarshalJSON(b []byte) error {`;
              $unmarshl.indented([
                `panic("UnmarshalJSON not implemented for NewType")`,
              ]);
              $unmarshl.add`}`;
              // todo marshal
              $marshall.add`func(v ${variantGoTypeIdent}) MarshalJSON() ([]byte, error) {`;
              $marshall.indented([
                `panic("MarshalJSON not implemented for NewType")`,
              ]);
              $marshall.add`}`;
              // typescript type
              $ns.lines.push(...variantDocs);
              $ns.add`export type ${variantGoTypeIdent} = {`;
              $ns.scope(($$) => {
                $$.add`${variantNameField}: {`;
                typeFieldsFinish$(createFormat, $$, fields);
              });
              $ns.add`}`;
              // create
              $ns.lines.push(...variantDocs);
              $ns.add`export function ${variantGoTypeIdent}(value: ${innerTypeRef}): ${variantGoTypeIdent} {`;
              $ns.ad1`return { ${variantNameField}: value }`;
              $ns.add`}`;
              // match
              $nsMatchToObj.lines.push(...variantDocs);
              $nsMatchToObj.add`${variantGoTypeIdent}(inner: ${innerTypeRef}): R,`;
              $nsMatchIfObjs.add`if (${namedStr(
                variant
              )} in input) return to.${variantGoTypeIdent}(input[${namedStr(
                variant
              )}]);`;
            },
          });
        }

        $decl.lines.push(...typeCode.lines);
      },
      NewTypeStruct(format) {
        const newTypeFormat = createFormat(format);
        // type
        $decl.lines.push(...docs);
        if (decl.serde_flags?.transparent) {
          $decl.add`export type ${goDeclIdent} = ${newTypeFormat.src}`;
        } else {
          $decl.add`export type ${goDeclIdent} = [${newTypeFormat.src}]`;
        }
        // create
        $decl.lines.push(...docs);
        $decl.add`export function ${goDeclIdent}(inner${
          newTypeFormat.optional && "?"
        }: ${newTypeFormat.src}): ${goDeclIdent} {`;
        if (decl.serde_flags?.transparent) {
          $decl.ad1`return inner;`;
        } else {
          $decl.ad1`return [inner];`;
        }
        $decl.add`}`;
      },
      TupleStruct(formats) {
        const formatTsList = tupleFormats(createFormat, formats);
        // type
        $decl.lines.push(...docs);
        $decl.add`export type ${goDeclIdent} = [${formatTsList
          .map((f) => f.fmt.src)
          .join(", ")}]`;
        // create
        $decl.lines.push(...docs);
        $decl.add`export function ${goDeclIdent}(${formatTsList
          .map((f) => `${f.id}: ${f.fmt.src}`)
          .join(", ")}): ${goDeclIdent} {`;
        $decl.ad1`return [${formatTsList.map((f) => f.id).join(", ")}];`;
        $decl.add`}`;
      },
      UnitStruct() {
        // type
        $decl.lines.push(...docs);
        $decl.add`export interface ${goDeclIdent} {} /* hmm unit struct? */`;
        // create
        $decl.lines.push(...docs);
        $decl.add`export function ${goDeclIdent}(): ${goDeclIdent} {`;
        $decl.ad1`return {};`;
        $decl.add`}`;
      },
    });

    generated.push({
      code: $decl,
      fileName: ident(decl.id) + "_gen.go",
      package: packageName,
    });
  }

  return generated;
}

function toOutput(generated: GeneratedDecl[]): gen.Output {
  return {
    errors: [],
    files: generated.map((gen) => ({
      path: `${gen.package}/wip/${gen.fileName}`,
      source: gen.code.toString(),
    })),
    warnings: [],
  };
}

function splitByFlattened<T extends gen.Attrs>(items: T[]) {
  const flattened: T[] = [];
  const fields: T[] = [];
  for (const item of items) {
    if (item.serde_flags?.flatten) {
      flattened.push(item);
    } else {
      fields.push(item);
    }
  }
  return { flattened, fields };
}

/** Only for the types */
function typeFieldsFinish$(
  createFormat: Formatter,
  root: Code,
  fields: gen.NamedField[]
) {
  const $ = root.indented();
  const split = splitByFlattened(fields);
  for (const field of split.fields) {
    const { src, optional } = createFormat(field.format);
    const isOptional =
      optional ||
      (field.serde_flags?.default && field.serde_attrs?.skip_serializing_if);
    $.addDocString(field);
    $.add`${namedField(field)}${isOptional && "?"}: ${src}${
      isOptional && " | null | undefined"
    };`;
  }

  if (split.flattened.length === 0) {
    root.add`};`;
    return;
  }
  root.add`} // flattened fields:`;
  for (const flattened of split.flattened) {
    root.addDocString(flattened, `Flattened from \`.${flattened.id}\`.`);
    const format = createFormat(flattened.format);
    if (format.optional) {
      root.add`& Partial<${format.src}>`;
    } else {
      root.add`& ${format.src}`;
    }
  }
  root.lastLine += ";";
}

function tupleFormats(ftr: Formatter, formats: gen.Format[]) {
  const aCp = "A".codePointAt(0)!;
  return formats.map((f, idx) => ({
    fmt: ftr(f),
    id: String.fromCodePoint(aCp + idx),
  }));
}

const num = () => ({ src: "number" });
const always = (src: string) => () => ({ src });

type Context = {
  addSupportingTuple(prefix: string, tuple: gen.Format[]): string;
};

type Formatter = (format: gen.Format) => {
  src: string;
  optional?: boolean;
};

const createFormatter =
  (context: Context): ((forIdent: string) => Formatter) =>
  (forIdent) => {
    const createFormat: Formatter = gen.Format.apply({
      TypeName: (value) => ({ src: ident(value.ident) }),
      I8: always("int8"),
      I16: always("int16"),
      I32: always("int32"),
      I64: always("int64"),
      I128: always("big.Int"),
      ISIZE: always("int"),
      U8: always("uint8"),
      U16: always("uint16"),
      U32: always("uint32"),
      U64: always("uint64"),
      U128: always("big.Int"),
      USIZE: always("uint"),
      F32: always("float32"),
      F64: always("float64"),
      Bool: () => ({ src: "bool" }),
      Bytes: () => ({ src: "/* bytes? */ []byte" }),
      Never: () => ({
        src: "/* Golang doesn't have a never type */ interface{}",
      }),
      Char: () => ({ src: "/* char */ byte" }),
      Map: ({ key, value }) => ({
        src: `map[${createFormat(key).src}]${createFormat(value).src}`,
      }),
      Unit: () => ({ src: "/* unit */ interface{}" }),
      Option: (format) => {
        const inner = createFormat(format);
        if (inner.optional) return inner;
        return {
          src: `*${inner.src}`,
          optional: true,
        };
      },
      Incomplete: ({ debug }) => ({
        src: `/* Incomplete: ${debug} */ interface{}`,
      }),
      Seq: (seq) => ({
        src: `[]${createFormat(seq).src}`,
      }),
      Tuple: (tuple) => ({
        src: context.addSupportingTuple(forIdent, tuple),
      }),
      TupleArray: ({ content, size }) => ({
        src: `[${size}]${createFormat(content).src}`,
      }),
      Str: () => ({ src: "string" }),
    });
    return createFormat;
  };

function ident(id: string): string {
  return id.replace(/[^a-zA-Z0-9\$\_]/g, "$").replace(/^(\d)/, "$$1");
}

function namedField(named: { id: string } & gen.Attrs): string {
  const nam = named.serde_attrs?.["rename"]?.[0] ?? named.id;
  if (/^[\w$][\w\d$]*$/.test(nam)) return nam;
  else return JSON.stringify(nam);
}

function namedStr(named: { id: string } & gen.Attrs): string {
  const nam = named.serde_attrs?.["rename"]?.[0] ?? named.id;
  return JSON.stringify(nam);
}

console.log(JSON.stringify(toOutput(convert(JSON.parse(Deno.args[0])))));
