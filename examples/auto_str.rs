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
    E33Test(MyEnum2),
}

#[derive(AutoStr)]
enum MyEnum4 {
    E41(MyEnum),
    E42(MyEnum2),
}

fn main() {
    check_try_from();
    check_to_string();
}

fn check_try_from() {
    assert!(matches!(MyEnum::try_from("e1"), Ok(MyEnum::E1)));
    assert!(matches!(MyEnum::try_from("E1"), Ok(MyEnum::E1)));
    assert!(matches!(MyEnum::try_from("e2"), Ok(MyEnum::E2)));
    assert!(matches!(MyEnum::try_from("ee"), Ok(MyEnum::E3)));
    assert!(matches!(MyEnum::try_from("e4"), Err(_)));

    assert!(matches!(MyEnum2::try_from("E21"), Ok(MyEnum2::E21)));
    assert!(matches!(
        MyEnum2::try_from("e2"),
        Ok(MyEnum2::E22(MyEnum::E2))
    ));
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

    assert!(matches!(
        MyEnum4::try_from("E1"),
        Ok(MyEnum4::E41(MyEnum::E1))
    ));
    assert!(matches!(MyEnum4::try_from("e1"), Err(_)));
}

fn check_to_string() {
    assert_eq!(MyEnum::E1.to_string(), "e1");
    assert_eq!(MyEnum2::E21.to_string(), "E21");
    assert_eq!(MyEnum2::E22(MyEnum::E1).to_string(), "e1");
    assert_eq!(MyEnum3::E31.to_string(), "E31");
    assert_eq!(MyEnum3::E32TesT.to_string(), "e32test");
    assert_eq!(MyEnum3::E33Test(MyEnum2::E22(MyEnum::E3)).to_string(), "e3");
}
