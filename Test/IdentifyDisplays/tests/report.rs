// What IdentifyDisplays prints, and its command line, which takes no
// arguments.

use displays::Display;
use identify_displays::{check_arguments, report};
use windows_api::{HMONITOR, RECT};

fn display(identifier: &str, name: &str, left: i32, primary: bool) -> Display {
    Display {
        identifier: identifier.to_string(),
        name: name.to_string(),
        connection: "DisplayPort",
        device_path: format!("\\\\?\\DISPLAY#AUSAA0F#{identifier}"),
        monitor: HMONITOR::default(),
        area: RECT {
            left,
            top: 0,
            right: left + 2560,
            bottom: 1440,
        },
        work_area: RECT {
            left,
            top: 0,
            right: left + 2560,
            bottom: 1392,
        },
        primary,
        dpi: 96,
        refresh_rate: Some(143.998),
    }
}

#[test]
fn each_display_gets_its_identifier_model_mode_place_and_device_path() {
    let displays = [display("9XH36C", "PA278QGV", -2560, false), display("CVVNY2", "PA278QGV", 0, true)];
    assert_eq!(
        report(&displays),
        "2 displays, from left to right:\n\
         \n\
         \x20 9XH36C  PA278QGV\n\
         \x20         2560 x 1440, 144 Hz, 100% scale, DisplayPort\n\
         \x20         At -2560, 0 on the desktop\n\
         \x20         \\\\?\\DISPLAY#AUSAA0F#9XH36C\n\
         \n\
         \x20 CVVNY2  PA278QGV, the primary display\n\
         \x20         2560 x 1440, 144 Hz, 100% scale, DisplayPort\n\
         \x20         At 0, 0 on the desktop\n\
         \x20         \\\\?\\DISPLAY#AUSAA0F#CVVNY2\n\
         \n\
         To open a program on a display, set \"Display\" in the Settings.json in its\n\
         folder to the display's identifier, or run it with --display and the identifier.\n"
    );
}

#[test]
fn one_display_is_one_display() {
    assert!(report(&[display("KXGR16", "PA278QGV", 0, true)]).starts_with("1 display, from left to right:\n"));
}

#[test]
fn no_displays_says_so() {
    assert_eq!(report(&[]), "Windows reports no displays.\n");
}

// Every display that Library/Displays finds is in the report, with its
// identifier and device path.
#[test]
fn the_report_has_every_display_that_windows_reports() {
    let displays = displays::enumerate().unwrap();
    assert!(!displays.is_empty());
    let text = report(&displays);
    for display in &displays {
        assert!(text.contains(&format!("  {}  {}", display.identifier, display.name)), "{text}");
        assert!(text.contains(&display.device_path), "{text}");
    }
}

#[test]
fn it_takes_no_arguments() {
    assert_eq!(check_arguments(&[]), Ok(()));
    let error = check_arguments(&["--list".to_string()]).unwrap_err();
    assert!(error.contains("--list"), "{error}");
}
