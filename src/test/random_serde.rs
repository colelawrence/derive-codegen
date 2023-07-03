use serde::Serialize;

/// Need to use serde_repr to use numbers
/// https://serde.rs/enum-number.html
#[derive(Serialize)]
#[repr(u8)]
enum EnumOfInts {
    A1 = 1,
    A2 = 2,
    C(usize),
}

/// Assert json inline with [`insta`]
macro_rules! assert_json {
    ($t:expr, @$t2:expr) => {
        insta::assert_snapshot!(serde_json::to_string(&$t).unwrap(), @$t2);
    };
}

#[test]
fn test_enum_of_ints() {
    assert_json!(EnumOfInts::A1, @r###""A1""###);
    assert_json!(EnumOfInts::A2, @r###""A2""###);
    assert_json!(EnumOfInts::C(14), @r###"{"C":14}"###);
}
