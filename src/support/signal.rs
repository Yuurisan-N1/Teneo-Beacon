fn farewell() {
    crate::support::log::restore_echo();
    crate::support::log::newline();
    crate::support::log::lr("Script stopped by user");
    std::process::exit(0);
}

#[cfg(unix)]
pub fn watch_interrupt() {
    tokio::spawn(async {
        let _ = tokio::signal::ctrl_c().await;
        farewell();
    });
}

#[cfg(windows)]
mod stop {
    #[link(name = "kernel32")]
    extern "system" {
        fn SetConsoleCtrlHandler(
            handler: Option<extern "system" fn(u32) -> i32>,
            add: i32,
        ) -> i32;
    }

    extern "system" fn on_console(_kind: u32) -> i32 {
        super::farewell();
        1
    }

    pub fn install() {
        unsafe {
            SetConsoleCtrlHandler(Some(on_console), 1);
        }
    }
}

#[cfg(windows)]
pub fn watch_interrupt() {
    stop::install();
}
