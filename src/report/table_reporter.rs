use super::{avg_median_str, memory_str, min_max_str, Reporter, ReporterClone};
use crate::bench::BenchResult;

#[derive(Clone, Copy)]
/// The TableReporter prints the results using prettytable.
///
/// It does not yet conver eveything, it does not report on OutputValue and perf stats.
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
