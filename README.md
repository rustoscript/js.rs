# js.rs [![Build Status](https://travis-ci.org/rustoscript/js.rs.svg)](https://travis-ci.org/rustoscript/js.rs)

A JavaScript interpreter written in Rust.

2015-2016 senior design project by Terry Sun and Saghm Rossi, in collaboration
with David Mally (for his Master's Thesis) and Professor Steve Zdancewic.

Our [poster] and [report] are included in PDF form under `report/` for
convenience.

  [poster]: https://github.com/rustoscript/js.rs/blob/master/report/poster.pdf
  [report]: https://github.com/rustoscript/js.rs/blob/master/report/report.pdf

## Structure

The parser is currently defined in a secondary package at
[rustoscript/js.rs-parser][parser], and the AST is defined at
[rustoscript/js.rs-common][common]. The garbage collection engine is implemented
by David Mally at [rustoscript/french-press][french-press].

  [parser]: https://github.com/rustoscript/js.rs-parser
  [common]: https://github.com/rustoscript/js.rs-common
  [french-press]: https://github.com/rustoscript/french-press
  [lalrpop]: https://github.com/nikomatsakis/lalrpop

## Instructions

Build and run with Cargo. By default, this opens a REPL.

```bash
cargo build
cargo run
```

You can pass in a JS file to evaluate code from a file.

```bash
cargo run <file>
```

Run the Sputnik test suite:

```bash
cargo run -- --test
```

## Evaluation

Js.rs was tested using Google's Sputnik, an ECMAScript 5 conformance test suite.
Sputnik defines several categories of tests, each with various depths of
subcategories (e.g., the "Expressions" category looks like this:

```
11_Expressions/
├── 11.3_PostfixExpressions
│   ├── 11.3.1_Postfix_Increment_Operator
│   │   ├── S11.3.1_A1.1_T1.js (a single test)
│   │   ├── S11.3.1_A1.1_T2.js
│   │   ├── ...
```

Overall, there are 111 leaf categories (categories which do not contain other
categories). We considered the number of leaf categories in which we had passed
at least one test. Of the 111 categories, we had coverage in 73 of the
categories, or 65.8%. This indicates that we covered a sizable portion of the
language's features. This metric represents the breadth of JavaScript that out
interpreter can handle.

Sputnik provides a total of 2427 distinct tests. Js.rs passes 18.2% of those
tests.
