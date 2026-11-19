// Program/RunLightly, run as coding agents run it. Each run that doesn't say
// otherwise makes a job of its own, as if no RunLightly ran it, though these
// tests run in one.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::Duration;

const RUN_LIGHTLY: &str = concat!(env!("CARGO_MANIFEST_DIR"), r"\..\..\Program\RunLightly\RunLightly.ps1");

// pwsh, running RunLightly.ps1 with the arguments, outside any RunLightly job
// of these tests'.
fn run_lightly() -> Command {
    let mut command = Command::new("pwsh");
    command.args(["-NoProfile", "-NonInteractive", "-File", RUN_LIGHTLY]).env_remove("KGAMES_LIGHT_JOB");
    command
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
