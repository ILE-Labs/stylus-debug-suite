const parser = require('@solidity-parser/parser');
const fs = require('fs');

const inputFile = process.argv[2];
if (!inputFile) {
    console.error('Usage: node parse_solidity.js <input_file>');
    process.exit(1);
}

try {
    const input = fs.readFileSync(inputFile, 'utf8');
    const ast = parser.parse(input, { loc: true, range: true });
    console.log(JSON.stringify(ast));
} catch (e) {
    if (e instanceof parser.ParserError) {
        console.error(JSON.stringify({ error: 'ParserError', details: e.errors }));
    } else {
        console.error(JSON.stringify({ error: 'Error', message: e.message }));
    }
    process.exit(1);
}
