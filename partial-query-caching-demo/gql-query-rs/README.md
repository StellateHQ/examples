# gql_query

_Stupendously fast and easy GraphQL Query Language handling._

The **gql_query** library follows two goals:

- To support a pleasant-to-use API for the GraphQL Query Language
- To be stupendously fast at processing GraphQL Query Language ASTs

In short, _surprise!_ The `gql_query` crate while handling a part of GraphQL does
not aim to support full, server-side GraphQL execution or the GraphQL Schema Language.
Many parts of the server-side execution of requests are one-off operations. Parsing a schema
and using it are operations that could even be preprocessed using the reference GraphQL.js
implementation.

A harder focus is to optimize how individual GraphQL requests are handled and making it easier
to write complex code on top of an easy to use AST.
GraphQL throughput is highly important in ensuring that GraphQL doesn't fall behind any other
solutions, which don't feature a rich query language.
On top, having an AST and library that's sufficiently easy to use and contains enough primitives
and utilities at the same time as valuing performance is something that's harder to do when
focusing on building a full GraphQL server.

As such, this library focuses on just processing GraphQL queries for the purpose of
intermediary GraphQL layers, which operate inbetween GraphQL clients and GraphQL servers.

[A good place to start learning more about this crate is the `ast` module...](src/ast/mod.rs)

## Sample Code

The library currently consists of utilities to parse, print, validate, visit, and transform
GraphQL Query Language ASTs.

```rust
use gql_query::{ast::*, validate::*};

let ctx = ASTContext::new();
let ast = Document::parse(&ctx, "{ field }").unwrap();

DefaultRules::validate(&ctx, &ast).unwrap()

let output = ast.print();
```

## Performance

We're aiming for at least 10x the performance and throughput of the
[GraphQL.js](https://github.com/graphql/graphql-js) reference implementation
(\*with V8's warmed JIT times) and most alternative libraries.

_How far along is this library in achieving that?_ Right on the mark.

```
test graphql_ast_fold     ... bench:         632 ns/iter (+/- 47)
test graphql_ast_parse    ... bench:       1,717 ns/iter (+/- 63)
test graphql_ast_print    ... bench:       1,072 ns/iter (+/- 25)
test graphql_ast_validate ... bench:       1,587 ns/iter (+/- 119)

gql-query-rs: (1000 iterations) 2.203µs
graphql-parser [rs]: (1000 iterations) 40.088µs
graphql.js: (10 iterations) 373.83µs
graphql.js: (100 iterations) 106.41µs
graphql.js: (1000 iterations) 33.33µs
```
