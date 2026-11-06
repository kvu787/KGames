// The harness: its statistics of given times, its budget, its table, its
// JSON, and the command line.

use benchmarks::{Budget, Measurement, Options, measure, parse_arguments, percentile, selected, table};

#[test]
fn the_statistics_are_of_the_times_per_unit() {
    let measurement = Measurement::from_times("a", "frame", &[2.0, 4.0, 6.0, 8.0, 10.0], 2.0);
    assert_eq!(measurement.iterations, 5);
    assert_eq!(measurement.min, 1.0);
    assert_eq!(measurement.median, 3.0);
    assert_eq!(measurement.mean, 3.0);
    assert_eq!(measurement.max, 5.0);
    assert!((measurement.percentile_99 - 4.96).abs() < 1.0e-9);
}

#[test]
fn a_percentile_is_between_the_two_nearest_values() {
    assert_eq!(percentile(&[], 0.5), 0.0);
    assert_eq!(percentile(&[7.0], 0.99), 7.0);
    assert_eq!(percentile(&[1.0, 3.0], 0.5), 2.0);
    assert_eq!(percentile(&[1.0, 2.0, 3.0, 4.0], 1.0), 4.0);
}

#[test]
fn the_budget_bounds_the_iterations() {
    let mut calls = 0;
    let measurement = measure("b", "call", Budget { seconds: 10.0, iterations: 7, warmup: 3 }, 1.0, || calls += 1);
    assert_eq!(calls, 10);
    assert_eq!(measurement.iterations, 7);
    let mut calls = 0;
    let measurement = measure("c", "call", Budget::ONCE, 1.0, || calls += 1);
    assert_eq!((calls, measurement.iterations), (1, 1));
    // The seconds stop it after the first iteration past them.
    let measurement = measure("d", "call", Budget { seconds: 0.0, iterations: 100, warmup: 0 }, 1.0, || {});
    assert_eq!(measurement.iterations, 1);
}

#[test]
fn the_table_has_a_row_a_measurement_with_its_note() {
    let measurement = Measurement::from_times("mesh/x", "subdivide", &[1.5], 1.0).with_note("6 faces");
    let text = table(&[measurement]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("Benchmark"));
    assert!(lines[1].starts_with("mesh/x"));
    assert!(lines[1].contains("1.5000"));
    assert!(lines[1].ends_with("6 faces"));
}

#[test]
fn a_measurement_s_json_has_every_statistic() {
    let value = Measurement::from_times("a", "frame", &[1.0, 2.0], 1.0).to_json();
    assert_eq!(value.get("Name"), Some(&json::Value::from("a")));
    assert_eq!(value.get("Iterations"), Some(&json::Value::from(2.0)));
    assert_eq!(value.get("MinMs"), Some(&json::Value::from(1.0)));
    assert_eq!(value.get("MaxMs"), Some(&json::Value::from(2.0)));
}

#[test]
fn the_command_line_picks_the_groups_the_budget_and_the_output() {
    let arguments: Vec<String> = ["--filter", "mesh", "--quick", "--json", "out.json"].iter().map(|s| s.to_string()).collect();
    assert_eq!(
        parse_arguments(&arguments),
        Ok(Options {
            filter: Some("mesh".to_string()),
            budget: Budget::QUICK,
            json: Some("out.json".to_string()),
            list: false,
        })
    );
    assert_eq!(parse_arguments(&[]).unwrap().budget, Budget::DEFAULT);
    assert!(parse_arguments(&["--filter".to_string()]).is_err());
    assert!(parse_arguments(&["--nope".to_string()]).is_err());
    assert_eq!(selected(Some("mesh")).len(), 1);
    assert_eq!(selected(Some("nothing")).len(), 0);
    assert_eq!(selected(None).len(), benchmarks::GROUPS.len());
}
