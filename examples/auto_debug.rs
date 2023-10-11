#![allow(dead_code)]
use std::fmt::{format, Formatter};

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

#[derive(AutoDebug)]
enum Foo3 {
    Foo1,
    Foo2((i32, u32)),
    Foo3(Foo2),
    Foo4 { a: i32, b: u32 },
}

#[derive(AutoDebug)]
enum Foo4<T: Sized + std::fmt::Debug, U>
where
    U: Sized + std::fmt::Debug,
{
    Foo1(T),
    Foo2(U),
    Foo3 { a: T, b: U },
}

#[derive(AutoDebug)]
struct Foo5<'a, T>
where
    T: Sized + std::fmt::Debug,
{
    foo1: &'a T,
}

fn main() {
    let foo1 = Foo1 {
        foo1: MyType {},
        foo2: MyType {},
        foo3: MyType {},
        foo4: MyType {},
    };

    assert_eq!(
        format(format_args!("{foo1:#?}")),
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

    assert_eq!(
        format(format_args!("{foo2:#?}")),
        r#"Foo2(
    debug MyType,
    display MyType,
)"#
    );

    let foo31 = Foo3::Foo1;
    assert_eq!(format(format_args!("{foo31:#?}")), r#""Foo1""#);

    let foo32 = Foo3::Foo2((-1, 2));
    assert_eq!(
        format(format_args!("{foo32:#?}")),
        r#"Foo2(
    (
        -1,
        2,
    ),
)"#
    );

    let foo33 = Foo3::Foo3(Foo2 {
        foo1: MyType {},
        foo2: MyType {},
    });
    assert_eq!(
        format(format_args!("{foo33:#?}")),
        r#"Foo3(
    Foo2(
        debug MyType,
        display MyType,
    ),
)"#
    );

    let foo34 = Foo3::Foo4 { a: -100, b: 200 };
    assert_eq!(
        format(format_args!("{foo34:#?}")),
        r#"{
    a: -100,
    b: 200,
}"#
    );

    let my_type = MyType {};
    let foo4 = Foo4::Foo3 { a: my_type, b: 4 };
    assert_eq!(
        format(format_args!("{foo4:#?}")),
        r#"{
    a: debug MyType,
    b: 4,
}"#
    );

    let my_type5 = MyType {};
    let foo5 = Foo5 { foo1: &my_type5 };
    assert_eq!(
        format(format_args!("{foo5:#?}")),
        r#"Foo5 {
    foo1: &'a debug MyType,
}"#
    );
}
