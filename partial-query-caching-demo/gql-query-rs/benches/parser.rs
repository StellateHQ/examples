#[macro_use]
extern crate bencher;

use bencher::Bencher;

fn graphql_ast_parse(bench: &mut Bencher) {
    use gql_query::ast::*;
    bench.iter(|| {
        let ctx = ASTContext::new();
        Document::parse(&ctx, QUERY).ok();
    });
}

fn graphql_ast_print(bench: &mut Bencher) {
    use gql_query::ast::*;
    let ctx = ASTContext::new();
    let ast = Document::parse(&ctx, QUERY).unwrap();
    bench.iter(|| ast.print());
}

fn graphql_ast_validate(bench: &mut Bencher) {
    use gql_query::ast::*;
    use gql_query::validate::*;
    let ctx = ASTContext::new();
    let ast = Document::parse(&ctx, QUERY).unwrap();
    bench.iter(|| ast.validate::<DefaultRules>(&ctx).unwrap());
}

fn graphql_ast_fold(bench: &mut Bencher) {
    use gql_query::ast::*;
    use gql_query::visit::*;

    #[derive(Default)]
    struct FoldNoop {}
    impl<'a> SimpleFolder<'a> for FoldNoop {
        fn named_type(&mut self, _name: NamedType<'a>) -> NamedType<'a> {
            NamedType { name: "oomph" }
        }
    }

    let ctx = ASTContext::new();
    let ast = Document::parse(&ctx, QUERY).unwrap();
    bench.iter(|| ast.fold(&ctx, &mut FoldNoop::default()).unwrap());
}

fn graphql_load_introspection(bench: &mut Bencher) {
    use gql_query::ast::ASTContext;
    use gql_query::schema::*;

    let ctx = ASTContext::new();

    bench.iter(|| {
        let introspection: IntrospectionQuery = serde_json::from_str(INTROSPECTION).unwrap();
        introspection.build_client_schema(&ctx);
    });
}

benchmark_group!(
    parse,
    graphql_ast_parse,
    graphql_ast_print,
    graphql_ast_validate,
    graphql_ast_fold,
    graphql_load_introspection,
);

benchmark_main!(parse);

static QUERY: &str = include_str!("../fixture/kitchen_sink.graphql");
static INTROSPECTION: &str = include_str!("../fixture/introspection_query.json");
