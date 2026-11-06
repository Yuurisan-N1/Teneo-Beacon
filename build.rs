fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        if let Err(err) = res.compile() {
            println!("cargo:warning=the Windows icon could not be attached: {}", err);
        }
    }
}
