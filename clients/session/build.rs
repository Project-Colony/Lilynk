//! Embed the Windows version-info + icon resources into `lilynk-session.exe`. The
//! icon drives Explorer, and `ly-presenter`'s `win32::stamp_window_icon` loads it by
//! ordinal 1 onto the SDL window's title bar / taskbar / Alt-Tab (SDL's own window-class
//! icon is the generic default).

fn main() {
    // No Linux leg: this binary carries no toolkit, so it has no gresource to compile. The
    // OS-mark icons belong to the GTK shell (`clients/linux/data`), which builds its own.

    // cfg(windows) is the HOST (skips Linux/macOS builds of this cross-platform binary);
    // CARGO_CFG_WINDOWS is the TARGET (x64 and cross-compiled ARM64 both pass).
    #[cfg(windows)]
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let icon = "../../packaging/windows/branding/lilynk.ico";
        println!("cargo:rerun-if-changed={icon}");
        winresource::WindowsResource::new()
            // Ordinal 1 — ly-presenter's win32.rs loads it by this id for WM_SETICON.
            .set_icon_with_id(icon, "1")
            // The version-info strings Windows shows for the bare exe (Properties→Details
            // lists FileDescription).
            .set("FileDescription", "Lilynk Session")
            .set("ProductName", "Lilynk")
            .set("CompanyName", "Lilynk")
            .compile()
            .expect("embed windows icon resource");
    }
}
