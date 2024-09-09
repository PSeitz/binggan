use super::{avg_median_str, memory_str, min_max_str, Reporter, ReporterClone};
use crate::bench::BenchResult;

#[derive(Clone, Copy)]
/// The TableReporter prints the results using prettytable.
///
/// It does not yet conver eveything, it does not report on OutputValue and perf stats.
///
/// ## Example
/// ```text
/// max id 100; 100 el all the same
/// | Name    | Memory         | Avg                   | Median                | Min .. Max                     |
/// +---------+----------------+-----------------------+-----------------------+--------------------------------+
/// | vec     | Memory: 404 B  | 8.6635 GiB/s (+1.16%) | 8.5639 GiB/s (-1.15%) | [8.7654 GiB/s .. 8.2784 GiB/s] |
/// | hashmap | Memory: 84 B   | 840.24 MiB/s (+1.54%) | 841.17 MiB/s (+0.33%) | [843.96 MiB/s .. 817.73 MiB/s] |
/// ```
pub struct TableReporter {}
impl ReporterClone for TableReporter {
    fn clone_box(&self) -> Box<dyn Reporter> {
        Box::new(*self)
    }
}
impl Reporter for TableReporter {
    fn report_results(&self, results: Vec<BenchResult>) {
        use prettytable::*;
        let mut table = Table::new();
        let format = format::FormatBuilder::new()
            .column_separator('|')
            .borders('|')
            .separators(
                &[format::LinePosition::Title],
                format::LineSeparator::new('-', '+', '+', '+'),
            )
            .padding(1, 1)
            .build();
        table.set_format(format);

        let mut row = prettytable::row!["Name", "Memory", "Avg", "Median", "Min .. Max"];
        if !results[0].tracked_memory {
            row.remove_cell(1);
        }
        table.set_titles(row);
        for result in results {
            let (avg_str, median_str) =
                avg_median_str(&result.stats, result.input_size_in_bytes, result.old_stats);
            let min_max = min_max_str(&result.stats, result.input_size_in_bytes);
            let memory_string = memory_str(&result.stats, result.old_stats, result.tracked_memory);
            let mut row = Row::new(vec![
                Cell::new(&result.bench_id.bench_name),
                Cell::new(&memory_string),
                Cell::new(&avg_str),
                Cell::new(&median_str),
                Cell::new(&min_max),
            ]);
            if !result.tracked_memory {
                row.remove_cell(1);
            }
            table.add_row(row);
        }
        table.printstd();
    }
}
