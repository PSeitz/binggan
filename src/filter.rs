use crate::bench_id::BenchId;
use tantivy_query_grammar::{Occur, UserInputAst, UserInputLeaf};

pub(crate) fn matches_filter(ast: &UserInputAst, bench_id: &BenchId) -> bool {
    match ast {
        UserInputAst::Clause(clauses) => {
            // A clause is a list of sub-ASTs with their Occur (Must, MustNot, Should)
            // By default, if there are Must clauses, all of them must match.
            // If there are only Should clauses, at least one must match.
            // MustNot clauses must not match.

            let mut has_must = false;
            let mut must_matches = true;

            let mut has_should = false;
            let mut should_matches = false;

            for (occur, sub_ast) in clauses {
                let m = matches_filter(sub_ast, bench_id);
                match occur {
                    Some(Occur::Must) => {
                        has_must = true;
                        if !m {
                            must_matches = false;
                        }
                    }
                    Some(Occur::MustNot) => {
                        if m {
                            return false; // Fast fail
                        }
                    }
                    Some(Occur::Should) | None => {
                        has_should = true;
                        if m {
                            should_matches = true;
                        }
                    }
                }
            }

            if has_must {
                must_matches
            } else if has_should {
                should_matches
            } else {
                true
            }
        }
        UserInputAst::Leaf(leaf) => {
            match &**leaf {
                UserInputLeaf::Literal(lit) => {
                    let field = lit.field_name.as_deref();
                    let phrase = &lit.phrase;
                    match field {
                        Some("r") | Some("runner_name") => bench_id
                            .runner_name
                            .as_deref()
                            .unwrap_or_default()
                            .contains(phrase),
                        Some("g") | Some("group_name") => bench_id
                            .group_name
                            .as_deref()
                            .unwrap_or_default()
                            .contains(phrase),
                        Some("b") | Some("bench_name") => bench_id.bench_name.contains(phrase),
                        _ => bench_id.get_full_name().contains(phrase),
                    }
                }
                UserInputLeaf::All => true,
                _ => false, // We only support Literals and All for now (no Regex, Range, etc)
            }
        }
        _ => false, // Boost etc. ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter() {
        let bench_id = BenchId::from_bench_name("my_bench")
            .runner_name(Some("my_runner"))
            .group_name(Some("my_group".to_string()));

        let queries = [
            ("my_bench", true),
            ("my_runner", true),
            ("my_group", true),
            ("my_bench AND my_runner", true),
            ("my_bench AND other_runner", false),
            ("my_bench OR other_runner", true),
            ("other_bench OR other_runner", false),
            ("bench_name:my_bench", true),
            ("bench_name:other_bench", false),
            ("group_name:my_group AND bench_name:my_bench", true),
            ("group_name:my_group bench_name:my_bench", true), // When no operator is specified, terms default to SHOULD. Both match here.
            ("group_name:my_group bench_name:other_bench", true), // With SHOULD terms, only one needs to match.
            ("my_bench NOT other_runner", true),
            ("my_bench -other_runner", true),
            ("my_bench NOT my_runner", false),
            ("my_bench -my_runner", false),
            ("NOT other_runner", true),
            ("-other_runner", true),
            ("NOT my_runner", false),
            ("-my_runner", false),
        ];

        for (query_str, expected) in queries {
            let ast = tantivy_query_grammar::parse_query(query_str).unwrap();
            assert_eq!(
                matches_filter(&ast, &bench_id),
                expected,
                "query: {}",
                query_str
            );
        }
    }
}
