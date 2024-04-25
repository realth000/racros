use std::io::Read;
use std::process::Stdio;

use racros::BundleText;

#[derive(BundleText)]
#[bundle(name = "some_file", file = "data/text")]
#[bundle(name = "my_rustc_version", command = "rustc --version")]
enum Bundler {}

fn main() {
    assert_eq!(
        Bundler::some_file(),
        r#"Some Text
To Read"#
    );
    let mut stdout = String::new();
    std::process::Command::new("rustc")
        .arg("--version")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .stdout
        .unwrap()
        .read_to_string(&mut stdout)
        .expect("failed to run rustc");
    assert_eq!(Bundler::my_rustc_version(), stdout);
}
