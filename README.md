# derive-codegen

WIP

'derive-codegen' allows you to build your own code generator based on rust enum and struct types.

Design:
 - Generation happens in two passes: 1. collect all the structural information from Rust code, 2. pass a
JSON of all that structural information and your custom attributes to your own
code generation command, which can be written in your preferred language.
 - You can "tag" your items in order to select which items should get generated for multiple code generators.
 - derive-codegen does minimal parsing and interpretation of serde attributes, and it is up to the codegen command to figure out what to do with aliases, rename, flatten, and others.

Interesting places in the codebase:
 - [Example generated internal types]( https://github.com/colelawrence/derive-codegen/blob/f22a9ea66919b73cc68b2a521916293b023077ea/typescript-generator/codegen-types/types.ts)
 - [xtask to generate those types](https://github.com/colelawrence/derive-codegen/blob/f22a9ea66919b73cc68b2a521916293b023077ea/xtask/src/generate_typescript_types.rs)
 - [Original rust types source](https://github.com/colelawrence/derive-codegen/blob/f22a9ea66919b73cc68b2a521916293b023077ea/src/generate.rs)
