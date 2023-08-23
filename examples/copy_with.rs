use racros::CopyWith;

#[derive(Clone, Default, CopyWith)]
struct MyStruct {
    foo1: i8,
    foo2: String,
    foo3: Option<String>,
}

#[derive(CopyWith)]
struct MyStruct2 {
    #[copy]
    bar1: MyStruct,
}

fn main() {
    let s1 = MyStruct::default();
    let mut s11 = MyStruct::default();
    let s2 = MyStruct {
        foo1: 64,
        foo2: String::from("hello world"),
        foo3: Some(String::from("hello world")),
    };
    let mut s21 = MyStruct {
        foo1: 64,
        foo2: String::from("hello world"),
        foo3: Some(String::from("hello world")),
    };

    s11.copy_with(&s2);
    assert_eq!(s11.foo1, s2.foo1);
    assert_eq!(s11.foo2, s2.foo2);
    assert_eq!(s11.foo3, s2.foo3);

    s21.copy_with(&s1);
    assert_eq!(s21.foo1, s2.foo1);
    assert_eq!(s21.foo2, s2.foo2);
    assert_eq!(s21.foo3, s2.foo3);

    let mut s31 = MyStruct2 {
        bar1: MyStruct::default(),
    };

    let s32 = MyStruct2 {
        bar1: MyStruct {
            foo1: 64,
            foo2: String::from("hello world"),
            foo3: Some(String::from("hello world")),
        },
    };

    s31.copy_with(&s32);
    assert_eq!(s31.bar1.foo1, s2.foo1);
    assert_eq!(s31.bar1.foo2, s2.foo2);
    assert_eq!(s31.bar1.foo3, s2.foo3);
}
