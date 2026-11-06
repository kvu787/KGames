// Every group's benchmarks run once, and measure something: a finite time
// above zero for each, with its name and unit.

use benchmarks::{Budget, GROUPS};

#[test]
fn every_group_runs_once_and_measures_something() {
    for group in GROUPS {
        let measurements = (group.run)(Budget::ONCE);
        assert!(!measurements.is_empty(), "{} measured nothing", group.name);
        for measurement in measurements {
            assert_eq!(measurement.iterations, 1, "{}", measurement.name);
            assert!(measurement.min.is_finite() && measurement.min >= 0.0, "{} took {} ms", measurement.name, measurement.min);
            assert!(measurement.min <= measurement.max, "{}", measurement.name);
            assert!(!measurement.unit.is_empty(), "{}", measurement.name);
            assert!(measurement.name.starts_with(group.name) || group.gpu, "{} isn't in {}", measurement.name, group.name);
        }
    }
}
