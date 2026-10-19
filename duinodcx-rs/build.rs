use std::io;

fn main() -> io::Result<()> {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        set_icon()?;
    }
    Ok(())
}

// winres is only a build-dependency on Windows hosts (see Cargo.toml),
// so Linux/Docker cross builds must not reference it.
#[cfg(windows)]
fn set_icon() -> io::Result<()> {
    let mut res = winres::WindowsResource::new();
    res.set_icon("Serial.ico");
    res.compile()
}

#[cfg(not(windows))]
fn set_icon() -> io::Result<()> {
    Ok(())
}
