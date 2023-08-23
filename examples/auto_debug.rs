use std::fmt::Formatter;

use racros::AutoDebug;

struct MyType {}

impl std::fmt::Debug for MyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("debug MyType")
    }
}

impl std::fmt::Display for MyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("display MyType")
    }
}

#[derive(AutoDebug)]
struct Foo1 {
    #[debug_name = "my_foo1"]
    foo1: MyType,
    #[debug_ignore]
    foo2: MyType,
    #[debug_display]
    foo3: MyType,
    #[debug_value = "foo4, MyType"]
    foo4: MyType,
}

#[derive(AutoDebug)]
#[debug_format = "display"]
#[debug_style = "tuple"]
struct Foo2 {
    #[debug_debug]
    foo1: MyType,
    foo2: MyType,
}

fn main() {
    let foo1 = Foo1 {
        foo1: MyType {},
        foo2: MyType {},
        foo3: MyType {},
        foo4: MyType {},
    };

    println!("{:#?}", foo1);

    assert_eq!(
        std::fmt::format(format_args!("{:#?}", foo1)),
        r#"Foo1 {
    my_foo1: debug MyType,
    foo3: display MyType,
    foo4: "foo4, MyType",
}"#
    );

    let foo2 = Foo2 {
        foo1: MyType {},
        foo2: MyType {},
    };

    println!("{:#?}", foo2);
    assert_eq!(
        std::fmt::format(format_args!("{:#?}", foo2)),
        r#"Foo2(
    debug MyType,
    display MyType,
)"#
    );
}
