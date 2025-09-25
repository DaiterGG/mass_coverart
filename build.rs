use {
    std::{env, io},
    winresource::WindowsResource,
};

fn main() -> io::Result<()> {
    if cfg!(windows) {
        println!("cargo:rustc-link-lib=Advapi32");
    }

    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("resources/icon_win.ico")
            .compile()?;
    }
    Ok(())
}
