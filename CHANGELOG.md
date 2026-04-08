Unreleased (2026-04-08)
==================
### Features
- Advanced filtering engine via `tantivy-query-grammar`
```
Replaced basic substring filtering with a powerful querying engine using tantivy-query-grammar.

Supports logical operators: AND, OR, NOT (and their symbols like -).
Supports field-based targeting:
- runner_name (alias: r)
- group_name (alias: g)
- bench_name (alias: b)

Enables granular selections, e.g., cargo bench -- "bench_name:my_bench AND group_name:my_group".
Keeps backward compatibility: basic strings fallback to substring matches on the full bench ID.
```
