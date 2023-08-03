package simple
import (
  "encoding/json"
  "fmt"
)

// supports
// `#[codegen(package = "simple", tags = "simple-go")]`
const todo = \`
export type SimpleStruct = {
  sstr: string;
  sint: int64;
};
// `#[codegen(package = "simple", tags = "simple-go")]`
export function SimpleStruct(inner: SimpleStruct): SimpleStruct {
  return inner;
}
\`