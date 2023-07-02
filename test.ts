// deno
const file = Deno.readFileSync("./src/generate.rs")
const dec = new TextDecoder()
console.log(dec.decode(file.slice(848, 853)))
