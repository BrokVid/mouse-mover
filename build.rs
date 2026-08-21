fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set("ProductName", "Mouse Mover");
    res.set("FileDescription", "Mouse Mover");
    res.set("OriginalFilename", "mouse-mover.exe");
    res.set("LegalCopyright", "Mouse Mover");
    res.set("ProductVersion", "0.1.0");
    res.set("FileVersion", "0.1.0");
    if let Err(err) = res.compile() {
        println!("cargo:warning=winresource failed: {err}");
    }
}
