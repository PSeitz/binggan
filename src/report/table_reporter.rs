use std::any::Any;

use yansi::Paint;

use super::{REPORTER_PLUGIN_NAME, avg_median_str, min_max_str};
use crate::{
    plugins::{EventListener, PluginEvents},
    report::{PrintOnce, check_and_print},
};

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
#[derive(Clone)]
pub struct TableReporter {
    print_runner_name_once: Option<PrintOnce>,
}

impl TableReporter {
    /// Creates a new TableReporter
    pub fn new() -> Self {
        Self {
            print_runner_name_once: None,
        }
    }
}

impl Default for TableReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventListener for TableReporter {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn name(&self) -> &'static str {
        REPORTER_PLUGIN_NAME
    }
    fn on_event(&mut self, event: PluginEvents) {
        match event {
            PluginEvents::BenchStart { bench_id: _ } => {}
            PluginEvents::GroupStart {
                runner_name,
                group_name,
                output_value_column_title: _,
            } => {
                if let Some(runner_name) = runner_name {
                    check_and_print(&mut self.print_runner_name_once, runner_name);
                }
                if let Some(group_name) = group_name {
                    println!("{}", group_name.black().on_yellow().invert().bold());
                }
            }
            PluginEvents::GroupStop {
                runner_name: _,
                group_name: _,
                results,
                output_value_column_title,
            } => {
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

                let mut headers = vec![Cell::new("Name")];
                for (key, _) in &results[0].formatted_custom_metrics {
                    headers.push(Cell::new(*key));
                }
                headers.push(Cell::new("Avg"));
                headers.push(Cell::new("Median"));
                headers.push(Cell::new("Min .. Max"));

                let has_output_value = results.iter().any(|r| r.output_value.is_some());
                if has_output_value {
                    headers.push(Cell::new(output_value_column_title));
                }
                table.set_titles(Row::new(headers));
                for result in results {
                    let (avg_str, median_str) = avg_median_str(
                        &result.stats,
                        result.input_size_in_bytes,
                        result.old_stats.as_ref(),
                    );
                    let min_max = min_max_str(&result.stats, result.input_size_in_bytes);

                    let mut row = vec![Cell::new(&result.bench_id.bench_name)];
                    for (_, formatted) in &result.formatted_custom_metrics {
                        row.push(Cell::new(formatted));
                    }
                    row.push(Cell::new(&avg_str));
                    row.push(Cell::new(&median_str));
                    row.push(Cell::new(&min_max));

                    if has_output_value {
                        row.push(Cell::new(
                            result.output_value.as_ref().unwrap_or(&"".to_string()),
                        ));
                    }
                    table.add_row(Row::new(row));
                }
                table.printstd();
            }
            _ => {}
        }
    }
}
