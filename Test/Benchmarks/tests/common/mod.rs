// Runs the tests of a test file that counts the whole process's heap, which
// runs without libtest, alone on the main thread. Like libtest, it takes
// cargo test's name filters, --exact, --ignored, which runs none, as none of
// them are ignored, and --list. Other options, like --nocapture, change
// nothing here.

use std::io::Write;

pub fn run(tests: &[(&str, fn())]) {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| arguments.iter().any(|argument| argument == name);
    let filters: Vec<&str> = arguments.iter().filter(|argument| !argument.starts_with('-')).map(String::as_str).collect();
    let chosen = |name: &str| {
        filters.is_empty() || filters.iter().any(|filter| if flag("--exact") { name == *filter } else { name.contains(filter) })
    };
    let to_run: Vec<&(&str, fn())> = tests.iter().filter(|(name, _)| chosen(name) && !flag("--ignored")).collect();
    if flag("--list") {
        for (name, _) in &to_run {
            println!("{name}: test");
        }
        return;
    }
    println!("running {} tests", to_run.len());
    // Stops at the first that fails, whose panic fails the run. Counting is
    // off between tests, so what this prints isn't counted.
    for (name, test) in &to_run {
        print!("test {name} ... ");
        std::io::stdout().flush().unwrap();
        test();
        println!("ok");
    }
    println!("test result: ok. {} passed; {} filtered out", to_run.len(), tests.len() - to_run.len());
}
