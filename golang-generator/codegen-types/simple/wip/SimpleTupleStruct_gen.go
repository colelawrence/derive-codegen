package simple
import (
  "encoding/json"
  "fmt"
)

// supports
// `#[codegen(package = "simple", tags = "simple-go")]`
export type SimpleTupleStruct = [uint8, big.Int, int]
// `#[codegen(package = "simple", tags = "simple-go")]`
export function SimpleTupleStruct(A: uint8, B: big.Int, C: int): SimpleTupleStruct {
  return [A, B, C];
}