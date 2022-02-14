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

#[derive(Builder, Debug, PartialEq)]
struct GenericStruct<T>(T);

#[test]
fn build_generic_struct() {
    let x = GenericStruct::<u32>::builder().set_0(33).build();
    assert_eq!(x, GenericStruct(33));
}

#[derive(Builder, Debug, PartialEq)]
struct ConstGeneric<T, const N: usize>([T; N]);

#[test]
fn build_const_generic() {
    let x = ConstGeneric::builder().set_0([32, 44, 61]).build();
    assert_eq!(x, ConstGeneric([32, 44, 61]));
}

#[derive(Builder, Debug, PartialEq)]
struct Nested {
    inner: FieldStruct,
}

#[test]
fn build_nested() {
    let x = Nested::builder()
        .build_inner()
        .set_name("emily".into())
        .set_value(127)
        .build()
        .build();
    assert_eq!(
        x,
        Nested {
            inner: FieldStruct {
                name: "emily".into(),
                value: 127,
            }
        }
    )
}

#[derive(Builder, Debug, PartialEq)]
struct WithDefaults {
    name: String,
    #[builder(default)]
    value: u32,
}

#[test]
fn build_defaults() {
    let x = WithDefaults::builder().set_name("emily".into()).build();
    assert_eq!(
        x,
        WithDefaults {
            name: "emily".into(),
            value: 0,
        }
    )
}
