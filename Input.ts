type LocationID = string;
export interface Input {
  declarations: InputDeclaration[];
}
interface InputDeclaration extends Attrs {
  id: string;
  id_location: LocationID;
  container_kind: ContainerFormat;
}
export interface Output {
  errors: OutputMessage[];
  warnings: OutputMessage[];
  files: OutputFile[];
}
interface OutputFile {
  path: string;
  source: string;
}
interface OutputMessage {
  message: string;
  labels: [string, LocationID][];
}
export type Format = { Incomplete: { debug: string; }; } |
{ TypeName: string; } |
{ Unit: null; } |
{ Bool: null; } |
{ I8: null; } |
{ I16: null; } |
{ I32: null; } |
{ I64: null; } |
{ I128: null; } |
{ ISIZE: null; } |
{ U8: null; } |
{ U16: null; } |
{ U32: null; } |
{ U64: null; } |
{ U128: null; } |
{ USIZE: null; } |
{ F32: null; } |
{ F64: null; } |
{ Char: null; } |
{ Str: null; } |
{ Bytes: null; } |
{ Option: Format; } |
{ Never: null; } |
{ Seq: Format; } |
{ Map: { key: Format; value: Format; }; } |
{ Tuple: Format[]; } |
{ TupleArray: { content: Format; size: number; }; };
type ContainerFormat = { UnitStruct: null; } |
{ NewTypeStruct: Format; } |
{ TupleStruct: Format[]; } |
{ Struct: { fields: NamedField[]; }; } |
{ Enum: { repr: EnumRepresentation; variants: NamedVariant[]; }; };
export interface NamedField extends Attrs {
  id: string;
  id_location: LocationID;
  format: Format;
}
interface NamedVariant extends Attrs {
  id: string;
  id_location: LocationID;
  variant_format: VariantFormat;
}
type VariantFormat = { Unit: null; } |
{ NewType: Format; } |
{ Tuple: Format[]; } |
{ Struct: { fields: NamedField[]; }; };
export interface Attrs {
  rust_docs: string | null;
  serde_attrs?: { [key: string]: [string, LocationID]; };
  serde_flags?: { [key: string]: LocationID; };
  codegen_attrs?: { [key: string]: [string, LocationID]; };
  codegen_flags?: { [key: string]: LocationID; };
}
type EnumRepresentation = { External: null; } |
{ Untagged: null; } |
{
  Tagged: {
    tag: string;
    tag_location: LocationID;
    content: string | null;
    content_location: LocationID | null;
  };
};
