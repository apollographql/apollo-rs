var { Source, parse } = require('graphql/language');

let input = process.argv[2];

let source = new Source(input);
parse(source);