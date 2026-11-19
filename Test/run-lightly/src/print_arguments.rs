// What the tests have RunLightly run: it prints its arguments, one a line,
// as Rust writes a string's value, quoted, and exits with the code in
// PRINT_ARGUMENTS_EXIT_CODE, or else 0.

fn main() {
    for argument in std::env::args().skip(1) {
        println!("{argument:?}");
    }
    let code = std::env::var("PRINT_ARGUMENTS_EXIT_CODE").map_or(0, |code| code.parse().unwrap());
    std::process::exit(code);
}
