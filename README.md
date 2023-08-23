# Racros

Collection of rust macros.

## Content

### AutoDebug

Generate debug trait implementation for structs with control.

#### Basic Usage

* `#[derive(AutoDebug)]` makes a struct style debug implementation.

#### Struct Attributes

* `#[debug_style = tuple]` makes a tuple style debug implementation. Default is struct style.
* `#[debug_format = display]` uses `Display` trait on fields. Default is debug format.

#### Struct Field Attributes

* `#[debug_name = "foo"]` override field name with "foo", if in struct `debug_style`.
* `#[debug_value = "foo"]` override field value with "foo".
* `#[debug_ignore]` will ignore this field in the output.
* `#[debug_debug]` will use [Debug] trait implementation for this field in output.
* `#[debug_display]` will use [Display] trait implementation for this field in output.

#### Example

For a custom type `MyType` that print `display MyType` in `Display` and `debug MyType` in `Debug`:

``` rust

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

```

Generate implementation with "{:#?}" for each field in `debug_struct` on default.

``` rust
#[derive(AutoDebug)]
struct Foo1 {
    #[debug_name = "my_foo1"] // Use "my_foo1" as strcut field name.
    foo1: MyType,
    #[debug_ignore] // Ignore this field.
    foo2: MyType,
    #[debug_display] // Use `Display` implementation for this field.
    foo3: MyType,
    #[debug_value = "foo4, MyType"] // Use "foo4, MyType" as struct field value.
    foo4: MyType,
}

#[derive(AutoDebug)]
#[debug_format = "display"] // Default implementation use "{}" for each field.
#[debug_style = "tuple"] // Output style set to `debug_tuple`.
struct Foo2 {
    #[debug_debug] // Use `Debug` implementation for this field.
    foo1: MyType,
    foo2: MyType,
}

let foo1 = Foo1 {
    foo1: MyType {},
    foo2: MyType {},
    foo3: MyType {},
    foo4: MyType {},
};

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

assert_eq!(
    std::fmt::format(format_args!("{:#?}", foo2)),
    r#"Foo2(
debug MyType,
display MyType,
)"#
    );

```

### AutoStr

Implement  `TryFrom`, `ToString` trait for enum types.

For the following code:

``` rust

#[derive(AutoStr)]
enum MyEnum {
    E1,
    E2,
    E3,
}

```

`AutoStr` will generate codes:

``` rust

impl TryFrom<&str> for MyEnum {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "E1" => Ok(MyEnum::E1),
            "E2" => Ok(MyEnum::E2),
            "E3" => Ok(MyEnum::E3),
            _ => Err(Self::Error::from(
                    format("failed to convert to {0} :invalid value","MyEnum")
                ))
        }
    }
}
impl ToString for MyEnum {
    fn to_string(&self) -> String {
        match self {
            MyEnum::E1 => "E1".to_string(),
            MyEnum::E2 => "E2".to_string(),
            MyEnum::E3 => "E3".to_string(),
        }
    }
}

```

The string format can be set to `lowercase`, `UPPERCASE`, `camelCase` or `PascalCase` by adding a `#[autorule = "xxxx"]`
attribute to the enum:

``` rust
#[derive(AutoStr)]
#[autorule = "lowercase"]
enum MyEnum {
    E1,
    E2,
    E3,
}

impl TryFrom<&str> for MyEnum {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "e1" => Ok(MyEnum::E1),
            "e2" => Ok(MyEnum::E2),
            "e3" => Ok(MyEnum::E3),
            _ => Err(Self::Error::from(
                    format("failed to convert to {} :invalid value","MyEnum")
                ))
        }
    }
}
impl ToString for MyEnum {
    fn to_string(&self) -> String {
        match self {
            MyEnum::E1 => "e1".to_string(),
            MyEnum::E2 => "e2".to_string(),
            MyEnum::E3 => "e3".to_string(),
        }
    }
}
```

In addition, adding the `#[str(...)]` attribute to enum field will override the default format.

``` rust
#[derive(AutoStr)]
#[autorule = "lowercase"]
enum MyEnum {
    #[str("e1", "E1")]
    E1,
    E2,
    #[str("e3", "ee")]
    E3,
}

impl TryFrom<&str> for MyEnum {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "e1" | "E1" => Ok(MyEnum::E1),
            "e2" => Ok(MyEnum::E2),
            "e3" | "ee"=> Ok(MyEnum::E3),
            _ => Err(Self::Error::from(
                    format("failed to convert to {} :invalid value","MyEnum")
                ))
        }
    }
}
impl ToString for MyEnum {
    fn to_string(&self) -> String {
        match self {
            MyEnum::E1 => "e1".to_string(),
            MyEnum::E2 => "e2".to_string(),
            MyEnum::E3 => "e3".to_string(),
        }
    }
}
```

Support embedded enums:

``` rust

enum MyEnum4 {
    E41(MyEnum),
    E42(MyEnum2),
}
impl TryFrom<&str> for MyEnum4 {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            _ => {
                let mut fallback_field: Option<&str> = None;
                let mut fallback_result: Option<Self> = None;
                if let Ok(v) = MyEnum::try_from(value) {
                    if fallback_result.is_some() {
                        return Err(
                            Self::Error::from({
                                format!(
                                    "#[str(...)] attribute not set and fallback guess is ambiguous: both {} and {} can accept this convert",
                                    fallback_field.unwrap(),
                                    "MyEnum",
                                )
                            }),
                        );
                    }
                    fallback_field = Some("MyEnum");
                    fallback_result = Some(MyEnum4::E41(v));
                }
                if let Ok(v) = MyEnum2::try_from(value) {
                    if fallback_result.is_some() {
                        return Err(
                            Self::Error::from({
                                format_args!(
                                    "#[str(...)] attribute not set and fallback guess is ambiguous: both {} and {} can accept this convert",
                                    fallback_field.unwrap(),
                                    "MyEnum2",
                                )
                            }),
                        );
                    }
                    fallback_field = Some("MyEnum2");
                    fallback_result = Some(MyEnum4::E42(v));
                }
                match fallback_result {
                    Some(v) => Ok(v),
                    None => {
                        Err(
                            Self::Error::from({
                                format_args!(
                                    "failed to convert to {} :invalid value",
                                    "MyEnum4",
                                )
                            }),
                        )
                    }
                }
            }
        }
    }
}
impl ToString for MyEnum4 {
    fn to_string(&self) -> String {
        match self {
            MyEnum4::E41(v) => v.to_string(),
            MyEnum4::E42(v) => v.to_string(),
        }
    }
}

```

### CopyWith

Similar to `copyWith` functions in dart, generates a function that allow copying data from another instance an override value in `self` if that value is not default value.

#### Basic Usage

For the following struct, generate:

``` rust

struct MyStruct {
    foo1: i8,
    foo2: String,
    foo3: Option<String>,
}

impl MyStruct {
     fn copy_with(&mut self, other: &Self) {
         if other.foo1 != i8::default() {
             self.foo1 = other.foo1.clone();
         }
         if other.foo2 != String::default() {
             self.foo2 = other.foo2.clone();
         }
         if other.foo3 != Option::default() {
             self.foo3 = other.foo3.clone();
         }
     }
 }

```

#### Field Attributes

Add `#[copy]` attribute to a field will try to call `.copy_with()` on that field instead of directly comparing values.

#### Example

``` rust

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
assert_eq!(s11.foo1, s2.foo1); // Copy s2.foo1 to s11.foo1
assert_eq!(s11.foo2, s2.foo2); // Copy s2.foo2 to s11.foo2
assert_eq!(s11.foo3, s2.foo3); // Copy s2.foo3 to s11.foo3

s21.copy_with(&s1);
assert_eq!(s21.foo1, s2.foo1); // Not copy because s1.foo1 is default value.
assert_eq!(s21.foo2, s2.foo2); // Not copy because s1.foo2 is default value.
assert_eq!(s21.foo3, s2.foo3); // Not copy because s1.foo3 is default value.

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

s31.copy_with(&s32); // Here use `s32.copy_with()` because bar1 has `#[copy]` attribute.
assert_eq!(s31.bar1.foo1, s2.foo1);
assert_eq!(s31.bar1.foo2, s2.foo2);
assert_eq!(s31.bar1.foo3, s2.foo3);

```
