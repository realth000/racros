use racros::BundleText;

#[derive(BundleText)]
#[bundle(name = "some_file", file = "data/text")]
#[bundle(name = "my_rustc_version", command = "rustc --version")]
enum Bundler {}

fn main() {}
