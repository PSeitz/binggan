Unreleased
==========
### Features
- Added `NUM_ITER_GROUP` environment variable to override benchmark group iterations

0.16.0 (2026-04-12)
===================
### Features
- Advanced filtering engine via `tantivy-query-grammar`
```
Extended basic substring filtering with a powerful querying engine using tantivy-query-grammar.

Supports logical operators: AND, OR, NOT (and their symbols like -).
Supports field-based targeting:
- runner_name (alias: r)
- group_name (alias: g)
- bench_name (alias: b)

Enables granular selections, e.g., cargo bench -- "bench_name:my_bench AND group_name:my_group".
You can also use the `BINGGAN_FILTER` environment variable to apply a filter.
Keeps backward compatibility: basic strings fallback to substring matches on the full bench ID.
```
