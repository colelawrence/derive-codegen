// deno
// args
let filepath = Deno.args[0] ?? "./src/generate.rs";
filepath = filepath.replace(/:\d+$/, "");
const denoByteArgs = Deno.args.slice(1).join(" ");
console.log("matching bytes from:", JSON.stringify(denoByteArgs));
let from = 100;
let to = 120;

const match2 = [/(\d+)[: ](\d+)/, /B(\d+)-B(\d+)/]
  .map((re) => re.exec(denoByteArgs))
  .find(a => a != null);
if (match2) {
  from = parseInt(match2[1]);
  to = parseInt(match2[2]);
} else {
  const match1 = /(\d+)/.exec(denoByteArgs);
  if (match1) {
    from = parseInt(match1[1]);
    to = from + 30;
  }
}

// parse
const file = Deno.readFileSync(filepath);
const dec = new TextDecoder();
console.log(filepath, { from, to });
console.log(
  "%c" +
    dec
      .decode(file.slice(0, from))
      .replace(/^[\s\S]+\n([^\n]*\n[^\n]*)$/, "$1") +
    "%c" +
    dec.decode(file.slice(from, to)) +
    "%c" +
    dec.decode(file.slice(to)).replace(/^([^\n]*\n[^\n]*)\n[\s\S]+$/, "$1"),
  "color: gray",
  "color: yellow",
  "color: gray"
);
