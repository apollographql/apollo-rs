// Regenerates the expected outputs for the graphql-js compatible printer tests
// (`crates/apollo-compiler/tests/graphql_js_printer.rs`) with a real graphql-js.
//
// Usage, from this directory:
//
//     npm install graphql@17.0.2
//     node generate.js ./node_modules/graphql [--raw] [out_dir]
//
// Inputs are every `.graphql` file in `../ok`, `../diagnostics` and `./input`.
// For each input that graphql-js can parse, `print(parse(input))` is written to
// `expected/<subdir>/<name>.graphql`. Inputs graphql-js cannot parse are listed
// in `expected/unsupported.txt` together with the error message.
//
// Unless `--raw` is given, the `block` flag of every `StringValue` node is first
// normalized to what apollo-compiler 1.x can represent (see README.md):
// `false` for values and `isPrintableAsBlockString(value)` for descriptions.
'use strict';
const fs = require('fs');
const path = require('path');

const [graphqlDir, ...rest] = process.argv.slice(2);
if (!graphqlDir) {
  console.error('usage: node generate.js <graphql module dir> [--raw] [out_dir]');
  process.exit(1);
}
const raw = rest.includes('--raw');
const outDir = rest.find((arg) => !arg.startsWith('--')) ?? path.join(__dirname, 'expected');
const graphql = require(path.resolve(graphqlDir));
const { isPrintableAsBlockString } = require(path.resolve(graphqlDir, 'language/blockString.js'));

const inputDirs = {
  ok: path.join(__dirname, '..', 'ok'),
  diagnostics: path.join(__dirname, '..', 'diagnostics'),
  input: path.join(__dirname, 'input'),
};

function normalizeBlockFlags(ast) {
  return graphql.visit(ast, {
    StringValue(node, key) {
      const block = key === 'description' ? isPrintableAsBlockString(node.value) : false;
      return { ...node, block };
    },
  });
}

const unsupported = [];
let written = 0;
for (const [subdir, dir] of Object.entries(inputDirs)) {
  const files = fs.readdirSync(dir).filter((f) => f.endsWith('.graphql')).sort();
  const outSubdir = path.join(outDir, subdir);
  fs.rmSync(outSubdir, { recursive: true, force: true });
  fs.mkdirSync(outSubdir, { recursive: true });
  for (const file of files) {
    const source = fs.readFileSync(path.join(dir, file), 'utf8');
    let ast;
    try {
      ast = graphql.parse(source);
    } catch (error) {
      unsupported.push(`${subdir}/${file}: ${error.message.split('\n')[0]}`);
      continue;
    }
    if (!raw) {
      ast = normalizeBlockFlags(ast);
    }
    fs.writeFileSync(path.join(outSubdir, file), graphql.print(ast));
    written += 1;
  }
}
fs.writeFileSync(path.join(outDir, 'unsupported.txt'), unsupported.map((l) => l + '\n').join(''));
console.log(`graphql@${graphql.version}: wrote ${written} expected outputs, ${unsupported.length} unsupported inputs`);
