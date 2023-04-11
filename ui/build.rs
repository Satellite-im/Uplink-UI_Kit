use glob::glob;
use rsass::{compile_scss, output};
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
};

fn main() -> Result<(), Box<dyn Error>> {
    let version = rustc_version::version().unwrap();
    // rustup install 1.68.2 will ensure that the compiler matches
    if version.major != 1 && version.minor != 68 && version.patch != 2 {
        panic!("rustc version != 1.68.2");
    }
    println!("cargo:rustc-env=RUSTC_VERSION={}", version);

    #[cfg(windows)]
    {
        //https://github.com/rust-lang/rfcs/blob/master/text/1665-windows-subsystem.md
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
        let mut res = winres::WindowsResource::new();
        res.set("ProductName", "uplink");
        res.set("FileDescription", "uplink");
        res.set(
            "LegalCopyright",
            "Creative Commons Attribution-NonCommercial 1.0",
        );
        res.set_icon("./extra/windows/uplink.ico");
        res.compile()
            .expect("Failed to run the Windows resource compiler (rc.exe)");
    }

    // Create the file that will hold the compiled CSS.
    let scss_output = "./src/compiled_styles.css";
    let mut scss = File::create(scss_output)?;

    // Create the string that will hold the concatenated contents of all SCSS files.
    let mut contents =
        String::from("/* This file is automatically generated, edits will be overwritten. */\n");

    // Use glob to read all SCSS files in the `src` directory and its subdirectories.
    let entries = glob("src/**/*.scss").map_err(|e| format!("Failed to read glob pattern: {e}"))?;

    // Concatenate the contents of each SCSS file into the `contents` string.
    for entry in entries {
        let path = entry?;
        let data = fs::read_to_string(path)?;
        contents += data.as_ref();
    }

    // Set the format for the compiled CSS.
    let format = output::Format {
        style: output::Style::Compressed,
        ..Default::default()
    };

    // Compile the SCSS string into CSS.
    let css = compile_scss(contents.as_bytes(), format)?;

    // Write the compiled CSS to the `scss` file.
    scss.write_all(&css)?;
    scss.flush()?;

    Ok(())
}
