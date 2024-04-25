//! # racros
//!
//! Racros is a collection of rust macros.
//!
//! ## [`AutoDebug`]
//!
//! Works on structs and enums, similar to [`std::fmt::Debug`] but support some customization:
//!
//! * Specify field name or value in print message.
//! * Ignore specified field.
//! * Use [`std::fmt::Display`] instead of [`std::fmt:;Debug`] on specified field.
//! * Set print style similar to printing tuple or struct.
//!
//! ## [`AutoStr`]
//!
//! Implement [`TryFrom`] `String` and [`ToString`] for enums with following features:
//!
//! * Specify what `String` value can convert from/to.
//! * Allow convert from multiple `String` values.
//! * Set default convert style: `lowercase`, `UPPERCASE`, `camelCase`, `PascalCase` and
//!   `snake_case`.
//!
//! ## [`CopyWith`]
//!
//! Add a `copy_with` function for decorated type, copy value from another `Self` if that value is
//! not `default` value.

////////////////////////////////////////////////////////////////////////////////

use proc_macro::TokenStream;

mod auto_debug;
mod auto_str;
mod bundle_text;
mod copy_with;
mod util;

/// Automatically add [`TryFrom`] `String` and [`ToString`] trait to the attached enum.
///
/// # Usage:
///   * `str`: add `#[str("str1")]` to field will add
/// the conversion from literal "str1" to that field
///   * Support using multiple str: `#str("str")]`.
///   * `#[autorule = "..." ]`, support autorules:
///     * `lowercase`.
///     * `UPPERCASE`.
///     * `camelCase`.
///     * `PascalCase`.
///     * `snake_case`.
///     * `SCREAMING_CASE`.
///
/// # Example:
///
/// ```
/// use racros::AutoStr;
///
/// #[derive(AutoStr, Debug)]
/// enum MyEnum {
///     #[str("e1", "E1")]
///     E1,
///     #[str("e2")]
///     E2,
///     #[str("e3", "ee")]
///     E3,
/// }
///
/// #[derive(AutoStr, Debug)]
/// enum MyEnum2 {
///     E21,
///     #[str("e1", "e2")]
///     E22(MyEnum),
/// }
///
/// #[derive(AutoStr, Debug)]
/// #[autorule = "lowercase"]
/// enum MyEnum3 {
///     #[str("E31")]
///     E31,
///     E32TesT,
///     #[str("e2")]
///     E33Test(MyEnum2),
/// }
///
/// #[derive(AutoStr, Debug)]
/// enum MyEnum4 {
///     E41(MyEnum),
///     E42(MyEnum2),
/// }
///
/// assert!(matches!(MyEnum::try_from("e1"), Ok(MyEnum::E1)));
/// assert!(matches!(MyEnum::try_from("E1"), Ok(MyEnum::E1)));
/// assert!(matches!(MyEnum::try_from("e2"), Ok(MyEnum::E2)));
/// assert!(matches!(MyEnum::try_from("ee"), Ok(MyEnum::E3)));
/// assert!(matches!(MyEnum::try_from("e4"), Err(_)));
///
/// assert!(matches!(MyEnum2::try_from("E21"), Ok(MyEnum2::E21)));
/// assert!(matches!(
///     MyEnum2::try_from("e1"),
///     Ok(MyEnum2::E22(MyEnum::E1))
/// ));
/// assert!(matches!(
///     MyEnum2::try_from("e2"),
///     Ok(MyEnum2::E22(MyEnum::E2))
/// ));
///
/// assert!(matches!(MyEnum3::try_from("E31"), Ok(MyEnum3::E31)));
/// assert!(matches!(MyEnum3::try_from("e32test"), Ok(MyEnum3::E32TesT)));
/// assert!(matches!(
///     MyEnum3::try_from("e2"),
///     Ok(MyEnum3::E33Test(MyEnum2::E22(MyEnum::E2)))
/// ));
///
/// assert_eq!(MyEnum::E1.to_string(), "e1");
/// assert_eq!(MyEnum2::E21.to_string(), "E21");
/// assert_eq!(MyEnum2::E22(MyEnum::E1).to_string(), "e1");
/// assert_eq!(MyEnum3::E31.to_string(), "E31");
/// assert_eq!(MyEnum3::E32TesT.to_string(), "e32test");
/// assert_eq!(MyEnum3::E33Test(MyEnum2::E22(MyEnum::E3)).to_string(), "e3");
/// assert!(matches!(
///     MyEnum4::try_from("E1"),
///     Ok(MyEnum4::E41(MyEnum::E1))
/// ));
///
/// assert_eq!(
///     MyEnum4::try_from("e1").unwrap_err(),
///     "#[str(...)] attribute not set and fallback guess is ambiguous: both MyEnum and MyEnum2 can accept this convert from \"e1\""
/// );
///
/// assert_eq!(
///     MyEnum4::try_from("e11").unwrap_err(),
///     "failed to convert to MyEnum4 :invalid value \"e11\""
/// );
///
/// ```
#[proc_macro_derive(AutoStr, attributes(str, autorule))]
pub fn auto_str(input: TokenStream) -> TokenStream {
    auto_str::auto_str_internal(input)
}

/// Add a `copy_with` function for decorated type, copy value from another `Self` if that value is
/// not `default` value.
///
///
/// For the following struct, generate:
///
/// ```
/// struct MyStruct {
///     foo1: i8,
///     foo2: String,
///     foo3: Option<String>,
/// }
///
/// impl MyStruct {
///      fn copy_with(&mut self, other: &Self) {
///          if other.foo1 != i8::default() {
///              self.foo1 = other.foo1.clone();
///          }
///          if other.foo2 != String::default() {
///              self.foo2 = other.foo2.clone();
///          }
///          if other.foo3 != Option::default() {
///              self.foo3 = other.foo3.clone();
///          }
///      }
///  }
///
/// ```
///
/// # Usage
///   * Add `#[derive(CopyWith)]` to struct.
///   * Because types and implementations are unknown in macro expanding, add `#[copy]` attribute
///     to the field which also `#[derived(CopyWith)]` so that will use that impl instead of default
///     value.
///   * Notice that the new value and cloned so all the fields can not be reference or borrowed type.
///
/// # Example:
///
/// ```
///
/// use racros::CopyWith;
/// #[derive(Clone, Default, CopyWith)]
/// struct MyStruct {
///     foo1: i8,
///     foo2: String,
///     foo3: Option<String>,
/// }
///
/// #[derive(CopyWith)]
/// struct MyStruct2 {
///     #[copy]
///     bar1: MyStruct,
/// }
///
/// let s1 = MyStruct::default();
/// let mut s11 = MyStruct::default();
/// let s2 = MyStruct {
///     foo1: 64,
///     foo2: String::from("hello world"),
///     foo3: Some(String::from("hello world")),
/// };
/// let mut s21 = MyStruct {
///     foo1: 64,
///     foo2: String::from("hello world"),
///     foo3: Some(String::from("hello world")),
/// };
///
/// s11.copy_with(&s2);
/// assert_eq!(s11.foo1, s2.foo1);
/// assert_eq!(s11.foo2, s2.foo2);
/// assert_eq!(s11.foo3, s2.foo3);
///
/// s21.copy_with(&s1);
/// assert_eq!(s21.foo1, s2.foo1);
/// assert_eq!(s21.foo2, s2.foo2);
/// assert_eq!(s21.foo3, s2.foo3);
///
/// let mut s31 = MyStruct2 {
///     bar1: MyStruct::default(),
/// };
///
/// let s32 = MyStruct2 {
///     bar1: MyStruct {
///         foo1: 64,
///         foo2: String::from("hello world"),
///         foo3: Some(String::from("hello world")),
///     },
/// };
///
/// s31.copy_with(&s32);
/// assert_eq!(s31.bar1.foo1, s2.foo1);
/// assert_eq!(s31.bar1.foo2, s2.foo2);
/// assert_eq!(s31.bar1.foo3, s2.foo3);
///
/// ```
///
#[proc_macro_derive(CopyWith, attributes(copy))]
pub fn copy_with(input: TokenStream) -> TokenStream {
    copy_with::copy_with_internal(input)
}

/// Generate debug trait implementation for structs and enums with control.
///
/// # Usage
///
///   * `#[derive(AutoDebug)]` makes a struct style debug implementation.
///
/// ## Struct Attributes
///
///   * `#[debug_style = tuple]` makes a tuple style debug implementation. Default is struct style.
///   * `#[debug_format = display]` uses `Display` trait on fields. Default is debug format.
///
/// ## Struct Field Attributes
///
///   * `#[debug_name = "foo"]` override field name with "foo", if in struct `debug_style`.
///   * `#[debug_value = "foo"]` override field value with "foo".
///   * `#[debug_ignore]` will ignore this field in the output.
///   * `#[debug_debug]` will use [Debug] trait implementation for this field in output.
///   * `#[debug_display]` will use `Display` trait implementation for this field in output.
///
/// ## Enum Variant Attributes
///   * `#[debug_ignore]`
///   * `#[debug_debug]`
///   * `#[debug_display]`
///
/// # Example
///
/// ```
/// use racros::AutoDebug;
/// use std::fmt::{format, Formatter};
///
/// struct MyType {}
///
/// impl std::fmt::Debug for MyType {
///     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///         f.write_str("debug MyType")
///     }
/// }
///
/// impl std::fmt::Display for MyType {
///     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///         f.write_str("display MyType")
///     }
/// }
///
/// // Struct
///
/// #[derive(AutoDebug)]
/// struct Foo1 {
///     #[debug_name = "my_foo1"]
///     foo1: MyType,
///     #[debug_ignore]
///     foo2: MyType,
///     #[debug_display]
///     foo3: MyType,
///     #[debug_value = "foo4, MyType"]
///     foo4: MyType,
/// }
///
/// #[derive(AutoDebug)]
/// #[debug_format = "display"]
/// #[debug_style = "tuple"]
/// struct Foo2 {
///     #[debug_debug]
///     foo1: MyType,
///     foo2: MyType,
/// }
///
/// let foo1 = Foo1 {
///     foo1: MyType {},
///     foo2: MyType {},
///     foo3: MyType {},
///     foo4: MyType {},
/// };
///
/// assert_eq!(
///     format(format_args!("{:#?}", foo1)),
///     r#"Foo1 {
///     my_foo1: debug MyType,
///     foo3: display MyType,
///     foo4: "foo4, MyType",
/// }"#
///  );
///
/// let foo2 = Foo2 {
///     foo1: MyType {},
///     foo2: MyType {},
/// };
///
/// assert_eq!(
///     format(format_args!("{:#?}", foo2)),
///     r#"Foo2(
///     debug MyType,
///     display MyType,
/// )"#
///     );
///
/// // Enum
///
/// #[derive(AutoDebug)]
/// enum Foo3 {
///     Foo1,
///     Foo2((i32, u32)),
///     Foo3(Foo2),
///     Foo4 { a: i32, b: u32 },
/// }
///
/// let foo33 = Foo3::Foo3(Foo2 {
///     foo1: MyType {},
///     foo2: MyType {},
/// });
/// assert_eq!(
///     format(format_args!("{:#?}", foo33)),
///     r#"Foo3(
///     Foo2(
///         debug MyType,
///         display MyType,
///     ),
/// )"#
/// );
///
/// let foo34 = Foo3::Foo4 { a: -100, b: 200 };
/// assert_eq!(
///     format(format_args!("{:#?}", foo34)),
///     r#"{
///     a: -100,
///     b: 200,
/// }"#
/// );
///
/// ```
///
#[proc_macro_derive(
    AutoDebug,
    attributes(
        debug_style,
        debug_format,
        debug_name,
        debug_value,
        debug_ignore,
        debug_debug,
        debug_display,
    )
)]
pub fn auto_debug(input: TokenStream) -> TokenStream {
    auto_debug::auto_debug_internal(input)
}

#[proc_macro_derive(BundleText, attributes(bundle))]
pub fn bundle_text(input: TokenStream) -> TokenStream {
    bundle_text::bundle_text_internal(input)
}
