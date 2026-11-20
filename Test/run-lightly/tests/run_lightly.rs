// Program/RunLightly, run as coding agents run it. Each run makes a job of its
// own, as if no RunLightly ran it, though these tests run in one, unless it
// says that it's in a job already.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::Duration;

const RUN_LIGHTLY: &str = concat!(env!("CARGO_MANIFEST_DIR"), r"\..\..\Program\RunLightly\RunLightly.ps1");
const PRINT_ARGUMENTS: &str = env!("CARGO_BIN_EXE_print-arguments");

// pwsh, writing its errors as plain text, without the colors that it adds in
// a terminal that says it takes them, like Git Bash's, whose TERM it gets,
// so that the tests read the same text from any shell.
fn pwsh() -> Command {
    let mut command = Command::new("pwsh");
    command.env("NO_COLOR", "1");
    command
}

// pwsh, running RunLightly.ps1 with the arguments, outside any RunLightly job
// of these tests'.
fn run_lightly() -> Command {
    let mut command = pwsh();
    command.args(["-NoProfile", "-NonInteractive", "-File", RUN_LIGHTLY]).env_remove("KGAMES_LIGHT_JOB");
    command
}

// pwsh, running the PowerShell, in a RunLightly job already, as far as
// RunLightly can tell, or not.
fn powershell(script: &str, in_a_job_already: bool) -> Command {
    let mut command = pwsh();
    command.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    if in_a_job_already {
        command.env("KGAMES_LIGHT_JOB", "unused");
    } else {
        command.env_remove("KGAMES_LIGHT_JOB");
    }
    command
}

// The text as a PowerShell string.
fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}

// The lines that PowerShell prints, before and after one that's ---.
fn halves(output: &Output) -> (Vec<String>, Vec<String>) {
    let printed = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{printed}{}", String::from_utf8_lossy(&output.stderr));
    let lines: Vec<String> = printed.lines().map(str::to_string).collect();
    let middle = lines.iter().position(|line| line == "---").expect(&printed);
    (lines[..middle].to_vec(), lines[middle + 1..].to_vec())
}

// Arguments that PowerShell reads in ways of its own: parameters written
// -name:value, which it gives a script as two, a -- that's quoted, as one
// that isn't is PowerShell's, quotes, an empty word, lists, a hashtable,
// which it doesn't enumerate, and $null, which it doesn't pass.
const ARGUMENTS: &str = r#"-p x -getProperty:Foo -oC:\out '-q:a' -n:5 -f:a,b '--' --flag /p:X=1 '' 'a b' 'say "hi"' x,y --format=%h,%s @{a=1} $null"#;

// What a program gets from ARGUMENTS, as print-arguments prints it.
fn printed_arguments() -> Vec<String> {
    [
        "-p", "x", "-getProperty:Foo", r"-oC:\out", "-q:a", "-n:5", "-f:a,b", "--", "--flag", "/p:X=1", "", "a b",
        r#"say "hi""#, "x,y", "--format=%h,%s", "System.Collections.Hashtable",
    ]
    .iter()
    .map(|argument| format!("{argument:?}"))
    .collect()
}

// A program gets its arguments from RunLightly as it would if PowerShell ran
// it itself: RunLightly once gave it -name:value as -name value, or only as
// value, and lists split into their items.
#[test]
fn a_program_gets_its_arguments_as_from_powershell() {
    for in_a_job_already in [false, true] {
        let program = quote(PRINT_ARGUMENTS);
        let script = format!("& {program} {ARGUMENTS}; '---'; & {} {program} {ARGUMENTS}; exit $LASTEXITCODE", quote(RUN_LIGHTLY));
        let (directly, through_run_lightly) = halves(&powershell(&script, in_a_job_already).output().unwrap());
        assert_eq!(directly, printed_arguments());
        assert_eq!(through_run_lightly, printed_arguments(), "in a job already: {in_a_job_already}");
    }
}

// A script gets its parameters from RunLightly as it would if PowerShell ran
// it itself: RunLightly once gave it -Name as a word, which went to its first
// parameter. Its exit code comes back.
#[test]
fn a_script_gets_its_parameters_as_from_powershell() {
    let script = Path::new(env!("CARGO_TARGET_TMPDIR")).join("run-lightly-parameters.ps1");
    std::fs::write(
        &script,
        "param([string]$Name, [int]$Count, [switch]$Loud)\r\n\
         Set-StrictMode -Version Latest\r\n\
         $ErrorActionPreference = 'Stop'\r\n\
         \"Name $Name, count $Count, loud $Loud\"\r\n\
         foreach ($argument in $args) { \"And $argument\" }\r\n\
         exit 7\r\n",
    )
    .unwrap();
    let script = quote(script.to_str().unwrap());
    let arguments = "-Name foo -Count 3 -Loud extra 'a b'";
    let expected = ["Name foo, count 3, loud True", "And extra", "And a b", "Exit code 7"];
    for in_a_job_already in [false, true] {
        let run = format!(
            "& {script} {arguments}; \"Exit code $LASTEXITCODE\"; '---'; & {} {script} {arguments}; \"Exit code $LASTEXITCODE\"",
            quote(RUN_LIGHTLY)
        );
        let (directly, through_run_lightly) = halves(&powershell(&run, in_a_job_already).output().unwrap());
        assert_eq!(directly, expected);
        assert_eq!(through_run_lightly, expected, "in a job already: {in_a_job_already}");
    }
}

// The command's output, once it ends and nothing holds its output open, or
// a failure after the time, which is long, as RunLightly runs at idle
// priority, and slowly under the job of these tests.
fn output_within(mut command: Command, time: Duration) -> Output {
    let child = command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || sender.send(child.wait_with_output()));
    match receiver.recv_timeout(time) {
        Ok(output) => output.unwrap(),
        Err(_) => panic!("{command:?}'s output didn't end within {time:?}: something holds it open"),
    }
}

// RunLightly exits with the command's exit code, whole, and says one that a
// byte can't hold, like an access violation's, in full, as coding agents'
// tools show only its low byte: 5.
#[test]
fn the_exit_code_comes_back_whole_and_a_crash_is_said_in_full() {
    for (code, said) in [(0, None), (3, None), (-1073741819, Some("RunLightly: the command exited with 0xC0000005"))] {
        let output = run_lightly().arg(PRINT_ARGUMENTS).env("PRINT_ARGUMENTS_EXIT_CODE", code.to_string()).output().unwrap();
        let errors = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(code), "{errors}");
        match said {
            Some(said) => assert!(errors.contains(said), "{errors}"),
            None => assert!(!errors.contains("RunLightly:"), "{errors}"),
        }
    }
}

// RunInBackground runs itself lightly. Run through RunLightly, it ended with
// "Access is denied" once it had built itself, as Windows didn't let it put
// the build and the program in RunLightly's job. It says to run it directly,
// before it builds anything.
#[test]
fn run_in_background_through_run_lightly_says_to_run_it_directly() {
    let run_in_background = concat!(env!("CARGO_MANIFEST_DIR"), r"\..\..\Program\RunInBackground\RunInBackground.ps1");
    let mut command = run_lightly();
    command.args([run_in_background, "minimal-scene-test"]);
    let output = output_within(command, Duration::from_secs(300));
    let errors = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{errors}");
    // PowerShell wraps an error's message in lines that start with |.
    let words = errors.split_whitespace().filter(|word| *word != "|").collect::<Vec<_>>().join(" ");
    assert!(words.contains(r"run Program\RunInBackground\RunInBackground.ps1 directly, not through RunLightly"), "{errors}");
    assert!(output.stdout.is_empty(), "{}", String::from_utf8_lossy(&output.stdout));
}

fn store_alias(name: &str) -> PathBuf {
    let alias = Path::new(&std::env::var("LOCALAPPDATA").unwrap()).join(r"Microsoft\WindowsApps").join(name);
    assert!(alias.exists(), "Windows 11 has {}", alias.display());
    alias
}

// winget's alias, which App Installer puts on every Windows 11, starts winget
// in a job of its own, which can't join RunLightly's. RunLightly ends it
// before it runs, and says why, instead of leaving it suspended, outside the
// job, where it would hold the output that this reads open forever.
#[test]
fn a_command_that_cant_join_the_job_is_ended_and_said_why() {
    let mut command = run_lightly();
    command.arg(store_alias("winget.exe")).arg("--version");
    let output = output_within(command, Duration::from_secs(300));
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{errors}");
    assert!(errors.contains("didn't let the command's process join RunLightly's job"), "{errors}");
    assert!(output.stdout.is_empty(), "{}", String::from_utf8_lossy(&output.stdout));
}
