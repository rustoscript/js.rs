# js.rs

A JavaScript interpreter written in Rust.

A senior design project by Terry Sun and Saghm Rossi, in collaboration with
David Mally and Professor Steve Zdancewic. Currently very in-progress.

## Structure

The parser and AST are currently defined in a secondary package at
[saghm/jsrs-parser][parser]. We are using [lalrpop][lalrpop] as our parser
generator.

  [parser]: https://github.com/saghm/jsrs-parser
  [lalrpop]: https://github.com/nikomatsakis/lalrpop
