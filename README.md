# Racros

Collection of rust macros.

## Content

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

### Rust Doc

``` rust
Automatically add [TryFrom] trait to the attached enum.

#### Usage:
  * `str`: add `#[str("str1")]` to field will add
the conversion from literal "str1" to that field
  * Support using multiple str: `#str("str")]`.
  * `#[autorule = "..." ]`, support autorule including `lowercase`, `UPPERCASE`, `camelCase` and `PascalCase`.

#### Example:

``` rust

use racros::AutoStr;

#[derive(AutoStr)]
enum MyEnum {
    #[str("e1", "E1")]
    E1,
    #[str("e2")]
    E2,
    #[str("e3", "ee")]
    E3,
}

#[derive(AutoStr)]
enum MyEnum2 {
    E21,
    #[str("e1", "e2")]
    E22(MyEnum),
}

#[derive(AutoStr)]
#[autorule = "lowercase"]
enum MyEnum3 {
    #[str("E31")]
    E31,
    E32TesT,
    #[str("e2")] // must have str attribute here because it has embedded enum
    E33Test(MyEnum2),
}

#[derive(AutoStr)]
enum MyEnum4 {
    E41(MyEnum),
    E42(MyEnum2),
}

assert!(matches!(MyEnum::try_from("e1"), Ok(MyEnum::E1)));
assert!(matches!(MyEnum::try_from("E1"), Ok(MyEnum::E1)));
assert!(matches!(MyEnum::try_from("e2"), Ok(MyEnum::E2)));
assert!(matches!(MyEnum::try_from("ee"), Ok(MyEnum::E3)));
assert!(matches!(MyEnum::try_from("e4"), Err(_)));

assert!(matches!(MyEnum2::try_from("E21"), Ok(MyEnum2::E21)));
assert!(matches!(
    MyEnum2::try_from("e1"),
    Ok(MyEnum2::E22(MyEnum::E1))
));
assert!(matches!(
    MyEnum2::try_from("e2"),
    Ok(MyEnum2::E22(MyEnum::E2))
));

assert!(matches!(MyEnum3::try_from("E31"), Ok(MyEnum3::E31)));
assert!(matches!(MyEnum3::try_from("e32test"), Ok(MyEnum3::E32TesT)));
assert!(matches!(
    MyEnum3::try_from("e2"),
    Ok(MyEnum3::E33Test(MyEnum2::E22(MyEnum::E2)))
));

assert_eq!(MyEnum::E1.to_string(), "e1");
assert_eq!(MyEnum2::E21.to_string(), "E21");
assert_eq!(MyEnum2::E22(MyEnum::E1).to_string(), "e1");
assert_eq!(MyEnum3::E31.to_string(), "E31");
assert_eq!(MyEnum3::E32TesT.to_string(), "e32test");
assert_eq!(MyEnum3::E33Test(MyEnum2::E22(MyEnum::E3)).to_string(), "e3");

assert!(matches!(
    MyEnum4::try_from("E1"),
    Ok(MyEnum4::E41(MyEnum::E1))
));
assert!(matches!(MyEnum4::try_from("e1"), Err(_)));

```
