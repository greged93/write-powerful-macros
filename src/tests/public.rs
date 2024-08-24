use public_macro::public;

#[test]
fn test_public_attribute() {
    #[public(exclude(name))]
    struct Person {
        name: String,
        age: u32,
    }

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };
}
