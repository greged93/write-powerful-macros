use config_macro::{config, config_struct};

#[test]
fn test_default_config_path() {
    config!();

    let config = Config::new();
    assert_eq!(config.get("user").unwrap(), "Iamtheuser");
    assert_eq!(config.get("pass").unwrap(), "Thisismypassword");
}

#[test]
fn test_config_with_path() {
    config!(path = "test-data/configuration/config.yaml");

    let config = Config::new();
    assert_eq!(config.get("user").unwrap(), "Iamtheotheruser");
    assert_eq!(config.get("pass").unwrap(), "Thisistheotherpassword");
}

#[test]
fn test_config_struct_default_path() {
    #[config_struct]
    struct PeopleAccess {
        other: u16,
        again_another_field: u128,
    }

    let people_access = PeopleAccess::new();
    assert_eq!(people_access.user, "Iamtheuser");
    assert_eq!(people_access.pass, "Thisismypassword");
}

#[test]
fn test_config_struct_with_path() {
    #[config_struct(path = "./test-data/configuration/config.yaml")]
    struct PeopleAccess {
        other: u16,
        again_another_field: u128,
    }

    let people_access = PeopleAccess::new();
    assert_eq!(people_access.user, "Iamtheotheruser");
    assert_eq!(people_access.pass, "Thisistheotherpassword");
}
