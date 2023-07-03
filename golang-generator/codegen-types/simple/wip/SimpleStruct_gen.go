package simple

/** `#[codegen(package = "simple", tags = "simple-go")]` */
export type SimpleStruct = {
  sstr: string;
  sint: number;
};
/** `#[codegen(package = "simple", tags = "simple-go")]` */
export function SimpleStruct(inner: SimpleStruct): SimpleStruct {
  return inner;
}