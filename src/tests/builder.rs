use builder_macro::Builder;

#[test]
fn test_base_builder_simple() {
    #[derive(Builder)]
    struct Person {
        name: String,
    }

    let builder = PersonBuilder::default();

    let person = builder.with_name("Alice".to_string()).build();
}

#[test]
fn test_base_builder() {
    #[derive(Builder)]
    struct Person {
        name: String,
        age: u32,
        kids: Vec<String>,
    }

    let builder = PersonBuilder::default();

    let person = builder
        .with_name("Alice".to_string())
        .with_age(30)
        .with_kids(vec!["Bob".to_string(), "Carol".to_string()])
        .build();
}

#[test]
fn test_rename_builder_function_attribute() {
    #[derive(Builder)]
    struct Person {
        name: String,
        age: u32,
        #[rename = "descendents"]
        kids: Vec<String>,
    }

    let builder = PersonBuilder::default();

    let person = builder
        .with_name("Alice".to_string())
        .with_age(30)
        .with_descendents(vec!["Bob".to_string(), "Carol".to_string()])
        .build();
}
