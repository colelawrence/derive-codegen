package simple
import (
  "encoding/json"
  "fmt"
)

// supports
type SimpleEnum_VTupleNestedTuple interface{} // TODO ["U8","I128","ISIZE"]
// `#[codegen(package = "simple", tags = "simple-go")]`
type SimpleEnum = SimpleEnumType

type SimpleEnumType interface{ isSimpleEnumType() }

// types
type SimpleEnum_VUnit struct{}
// awdji VUnit2 has docs
type SimpleEnum_VUnit2 struct{}
type SimpleEnum_VStr string
type SimpleEnum_VStr2 string
// lkahal VNewTypeStruct has docs
type SimpleEnum_VNewTypeStruct SimpleStruct
type SimpleEnum_VTuple struct {
  // todo tuple fields like A, B, C
  A string
  B int64
}
type SimpleEnum_VTupleNested struct {
  // todo tuple fields like A, B, C
  A string
  B SimpleEnum_VTupleNestedTuple
}
// 90uw8d VStruct variant has docs
type SimpleEnum_VStruct struct {
  // todo shared fields def
}
// impl
func (SimpleEnum_VUnit) isSimpleEnumType() {}
func (SimpleEnum_VUnit2) isSimpleEnumType() {}
func (SimpleEnum_VStr) isSimpleEnumType() {}
func (SimpleEnum_VStr2) isSimpleEnumType() {}
func (SimpleEnum_VNewTypeStruct) isSimpleEnumType() {}
func (SimpleEnum_VTuple) isSimpleEnumType() {}
func (SimpleEnum_VTupleNested) isSimpleEnumType() {}
func (SimpleEnum_VStruct) isSimpleEnumType() {}
// unmarshal
func(v *SimpleEnum_VUnit) UnmarshalJSON(b []byte) error {
  panic("UnmarshalJSON not implemented for NewType")
}
func(v *SimpleEnum_VUnit2) UnmarshalJSON(b []byte) error {
  panic("UnmarshalJSON not implemented for NewType")
}
func(v *SimpleEnum_VStr) UnmarshalJSON(b []byte) error {
  panic("UnmarshalJSON not implemented for NewType")
}
func(v *SimpleEnum_VStr2) UnmarshalJSON(b []byte) error {
  panic("UnmarshalJSON not implemented for NewType")
}
func(v *SimpleEnum_VNewTypeStruct) UnmarshalJSON(b []byte) error {
  panic("UnmarshalJSON not implemented for NewType")
}
func(v *SimpleEnum_VTuple) UnmarshalJSON(b []byte) error {
  panic("UnmarshalJSON not implemented for NewType")
}
func(v *SimpleEnum_VTupleNested) UnmarshalJSON(b []byte) error {
  panic("UnmarshalJSON not implemented for NewType")
}
func(v *SimpleEnum_VStruct) UnmarshalJSON(b []byte) error {
  panic("UnmarshalJSON not implemented for NewType")
}
// marshal
func(v SimpleEnum_VUnit) MarshalJSON() ([]byte, error) {
  panic("MarshalJSON not implemented for NewType")
}
func(v SimpleEnum_VUnit2) MarshalJSON() ([]byte, error) {
  panic("MarshalJSON not implemented for NewType")
}
func(v SimpleEnum_VStr) MarshalJSON() ([]byte, error) {
  panic("MarshalJSON not implemented for NewType")
}
func(v SimpleEnum_VStr2) MarshalJSON() ([]byte, error) {
  panic("MarshalJSON not implemented for NewType")
}
func(v SimpleEnum_VNewTypeStruct) MarshalJSON() ([]byte, error) {
  panic("MarshalJSON not implemented for NewType")
}
func(v SimpleEnum_VTuple) MarshalJSON() ([]byte, error) {
  panic("MarshalJSON not implemented for NewType")
}
func(v SimpleEnum_VTupleNested) MarshalJSON() ([]byte, error) {
  panic("MarshalJSON not implemented for NewType")
}
func(v SimpleEnum_VStruct) MarshalJSON() ([]byte, error) {
  panic("MarshalJSON not implemented for NewType")
}
// `#[codegen(package = "simple", tags = "simple-go")]`
// deno-lint-ignore no-namespace
export namespace SimpleEnum {
  export type ApplyFns<R> = {
    // callbacks
    SimpleEnum_VUnit(): R,
    // awdji VUnit2 has docs
    SimpleEnum_VUnit2(): R,
    SimpleEnum_VStr(inner: SimpleEnum_VStr["VStr"]): R;
    SimpleEnum_VStr2(inner: SimpleEnum_VStr2["VStr2"]): R;
    // lkahal VNewTypeStruct has docs
    SimpleEnum_VNewTypeStruct(inner: SimpleEnum_VNewTypeStruct["VNewTypeStruct"]): R;
    SimpleEnum_VTuple(inner: [string, int64]): R,
    SimpleEnum_VTupleNested(inner: [string, SimpleEnum_VTupleNestedTuple]): R,
    // 90uw8d VStruct variant has docs
    SimpleEnum_VStruct(inner: SimpleEnum_VStruct["VStruct"]): R,
  }
  /** Match helper for {@link SimpleEnum} */
  export function apply<R>(
    to: ApplyFns<R>,
  ): (input: SimpleEnum) => R {
    return function _match(input): R {
      // if-else strings
      if (input === "VUnit") return to.SimpleEnum_VUnit();
      if (input === "VUnit2") return to.SimpleEnum_VUnit2();
      // if-else objects
      if (typeof input !== "object" || input == null) throw new TypeError("Unexpected non-object for input");
      if ("VStr" in input) return to.VStr(input["VStr"]);
      if ("VStr2" in input) return to.VStr2(input["VStr2"]);
      if ("VNewTypeStruct" in input) return to.VNewTypeStruct(input["VNewTypeStruct"]);
      if ("VTuple" in input) return to.SimpleEnum_VTuple(input["VTuple"]);
      if ("VTupleNested" in input) return to.SimpleEnum_VTupleNested(input["VTupleNested"]);
      if ("VStruct" in input) return to.SimpleEnum_VStruct(input["VStruct"]);
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
  export type SimpleEnum_VUnit = "VUnit"
  export function SimpleEnum_VUnit(): SimpleEnum_VUnit {
    return "VUnit";
  }
  // awdji VUnit2 has docs
  export type SimpleEnum_VUnit2 = "VUnit2"
  // awdji VUnit2 has docs
  export function SimpleEnum_VUnit2(): SimpleEnum_VUnit2 {
    return "VUnit2";
  }
  export function SimpleEnum_VStr(value: string): SimpleEnum_VStr {
    return { VStr: value };
  }
  export function SimpleEnum_VStr2(value: string): SimpleEnum_VStr2 {
    return { VStr2: value };
  }
  // lkahal VNewTypeStruct has docs
  export function SimpleEnum_VNewTypeStruct(value: SimpleStruct): SimpleEnum_VNewTypeStruct {
    return { VNewTypeStruct: value };
  }
  export type SimpleEnum_VTuple = { VTuple: [string, int64] };
  export function SimpleEnum_VTuple(A: string, B: int64): SimpleEnum_VTuple {
    return { VTuple: [A, B] };
  }
  export type SimpleEnum_VTupleNested = { VTupleNested: [string, SimpleEnum_VTupleNestedTuple] };
  export function SimpleEnum_VTupleNested(A: string, B: SimpleEnum_VTupleNestedTuple): SimpleEnum_VTupleNested {
    return { VTupleNested: [A, B] };
  }
  // 90uw8d VStruct variant has docs
  export type SimpleEnum_VStruct = {
    VStruct: {
      vfield: string;
    };
  }
  // 90uw8d VStruct variant has docs
  export function SimpleEnum_VStruct(value: SimpleEnum_VStruct["VStruct"]): SimpleEnum_VStruct {
    return { VStruct: value }
  }
}
// `#[codegen(package = "simple", tags = "simple-go")]`
export type SimpleEnum =
  | SimpleEnum.SimpleEnum_VUnit
  | SimpleEnum.SimpleEnum_VUnit2
  | SimpleEnum.SimpleEnum_VStr
  | SimpleEnum.SimpleEnum_VStr2
  | SimpleEnum.SimpleEnum_VNewTypeStruct
  | SimpleEnum.SimpleEnum_VTuple
  | SimpleEnum.SimpleEnum_VTupleNested
  | SimpleEnum.SimpleEnum_VStruct