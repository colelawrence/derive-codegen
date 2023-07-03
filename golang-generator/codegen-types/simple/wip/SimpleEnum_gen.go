package simple

/** `#[codegen(package = "simple", tags = "simple-go")]` */
// deno-lint-ignore no-namespace
export namespace SimpleEnum {
  export type ApplyFns<R> = {
    // callbacks
    VUnit(): R,
    VStr(inner: VStr["VStr"]): R;
    VTuple(inner: [string, number]): R,
    VStruct(inner: VStruct["VStruct"]): R,
  }
  /** Match helper for {@link SimpleEnum} */
  export function apply<R>(
    to: ApplyFns<R>,
  ): (input: SimpleEnum) => R {
    return function _match(input): R {
      // if-else strings
      if (input === "VUnit") return to.VUnit();
      // if-else objects
      if (typeof input !== "object" || input == null) throw new TypeError("Unexpected non-object for input");
      if ("VStr" in input) return to.VStr(input["VStr"]);
      if ("VTuple" in input) return to.VTuple(input["VTuple"]);
      if ("VStruct" in input) return to.VStruct(input["VStruct"]);
      const _exhaust: never = input;
      return _exhaust;
    }
  }
  /** Match helper for {@link SimpleEnum} */
  export function match<R>(
    input: SimpleEnum,
    to: ApplyFns<R>,
  ): R {
    return apply(to)(input)
  }
  export type VUnit = "VUnit"
  export function VUnit(): VUnit {
    return "VUnit";
  }
  export type VStr = {
    VStr: string
  };
  export function VStr(value: string): VStr {
    return { VStr: value };
  }
  export type VTuple = { VTuple: [string, number] };
  export function VTuple(a: string, b: number): VTuple {
    return { VTuple: [a, b] };
  }
  export type VStruct = {
    VStruct: {
      vfield: string;
    };
  }
  export function VStruct(value: VStruct["VStruct"]): VStruct {
    return { VStruct: value }
  }
}
/** `#[codegen(package = "simple", tags = "simple-go")]` */
export type SimpleEnum =
  | SimpleEnum.VUnit
  | SimpleEnum.VStr
  | SimpleEnum.VTuple
  | SimpleEnum.VStruct