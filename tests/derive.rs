use builder::Builder;

#[derive(Builder, Debug, PartialEq)]
struct UnitStruct;

#[test]
fn build_unit_struct() {
    let x = UnitStruct::builder().build();
    assert_eq!(x, UnitStruct);
}

#[derive(Builder, Debug, PartialEq)]
struct TupleStruct(String, u32);

#[test]
fn build_tuple_struct() {
    let x = TupleStruct::builder()
        .set_0("hello".into())
        .set_1(42)
        .build();
    assert_eq!(x, TupleStruct("hello".into(), 42));
}

#[derive(Builder, Debug, PartialEq)]
struct FieldStruct {
    name: String,
    value: u32,
}

#[test]
fn build_field_struct() {
    let x = FieldStruct::builder()
        .set_name("emily".into())
        .set_value(17)
        .build();
    assert_eq!(
        x,
        FieldStruct {
            name: "emily".into(),
            value: 17
        }
    );
}
