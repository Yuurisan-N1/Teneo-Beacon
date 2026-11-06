pub enum Mode {
    Run,
    Probe,
    SelfTest,
}

pub fn parse() -> Mode {
    match std::env::args().nth(1).unwrap_or_default().as_str() {
        "probe" => Mode::Probe,
        "selftest" => Mode::SelfTest,
        "" | "run" | "loop" => Mode::Run,
        other => {
            crate::support::log::ly(&format!(
                "Unknown mode {} was requested so the loop is used",
                other
            ));
            Mode::Run
        }
    }
}
