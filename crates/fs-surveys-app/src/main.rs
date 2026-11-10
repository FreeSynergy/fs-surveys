//! `fs-surveys`: opens the surveys window.
//!
//! Pattern: none — the command line only calls `gui::run`.

fn main() {
    let words: Vec<String> = std::env::args().skip(1).collect();
    if let Err(message) = fs_surveys_app::gui::run(&words) {
        eprintln!("fs-surveys: {message}");
        std::process::exit(1);
    }
}
