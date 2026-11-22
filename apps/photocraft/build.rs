//! Windows only: embed the app icon and version info (VERSIONINFO) into `photocraft.exe`.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `PHOTOCRAFT_REQUIRE_WINRES=1` (set by the
//! release workflow) turns it into an error.

/// Embeds the Windows resources and delay-loads the print-only system DLLs (#5884, #8272).
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/photocraft.ico");
    println!("cargo:rerun-if-env-changed=PHOTOCRAFT_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    // Printing to PDF (winprint's `PdfCreateRenderer`) imports Windows.Data.Pdf.dll, and its
    // printer options (the Print Ticket API) import prntvpt.dll; stripped Windows images (NTLite,
    // some N/LTSC builds) lack them. A load-time import stops the exe from starting at all
    // ("windows.data.pdf.dll was not found", #5884; "prntvpt.dll", #8272), so load them on first
    // use; the print commands check for them first and report an error instead.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg-bins=/DELAYLOAD:windows.data.pdf.dll");
        println!("cargo:rustc-link-arg-bins=/DELAYLOAD:prntvpt.dll");
        println!("cargo:rustc-link-arg-bins=delayimp.lib");
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/photocraft.ico")
        .set("ProductName", "PhotoCraft")
        .set("FileDescription", "PhotoCraft image editor")
        .set("CompanyName", "Learning Machines LLC")
        .set("LegalCopyright", "Copyright (c) the PhotoCraft authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "photocraft.exe")
        .set("InternalName", "photocraft");
    // MinGW/GNU `ld` ignores MSVC `/MANIFESTDEPENDENCY:` linker directives from crates that
    // import ComCtl32 v6 symbols (`TaskDialog`, `TaskDialogIndirect`), so embed the v6 manifest
    // directly for GNU builds.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
        res.set_manifest(
            r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
      <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}"/>
      <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}"/>
      <supportedOS Id="{e2011457-1546-43c5-a5fe-008deee3d3f0}"/>
    </application>
  </compatibility>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
</assembly>"#,
        );
    }
    if let Err(e) = res.compile() {
        if std::env::var_os("PHOTOCRAFT_REQUIRE_WINRES").is_some() {
            panic!("embedding Windows resources failed: {e}");
        }
        println!("cargo:warning=photocraft.exe built without icon/version resources: {e}");
    }
}
