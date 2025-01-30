use std::{env, fs, io::stdout, process};

use crate::models::*;

pub fn print_env() {
    print_list_as_variables(&mut stdout(), env::vars().collect());
}

fn print_list_as_variables(writer: &mut dyn std::io::Write, variables: Vec<(String, String)>) {
    for (key, value) in variables {
        writeln!(writer, "{} = \"{}\"", key, value).expect("can't write to buffer");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_list_as_variables() {
        let mut writer = Vec::new();
        let variables = vec![
            ("MY_ENV_VAR".to_string(), "TEST".to_string()),
            ("TEST".to_string(), "hello".to_string()),
        ];
        print_list_as_variables(&mut writer, variables);
        let result = String::from_utf8(writer).unwrap();
        assert_eq!(result, "MY_ENV_VAR = \"TEST\"\nTEST = \"hello\"\n");
    }

    #[test]
    fn test_print_list_as_variables_with_empty_vector() {
        let mut writer = Vec::new();
        let variables = Vec::new();
        print_list_as_variables(&mut writer, variables);
        let result = String::from_utf8(writer).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_print_list_as_variables_with_special_characters() {
        let mut writer = Vec::new();
        let variables = vec![
            ("SPECIAL_CHAR".to_string(), "value_with_@_#_$".to_string()),
            ("ANOTHER_SPECIAL".to_string(), "value_with_!_&_*".to_string()),
        ];
        print_list_as_variables(&mut writer, variables);
        let result = String::from_utf8(writer).unwrap();
        assert_eq!(result, "SPECIAL_CHAR = \"value_with_@_#_$\"\nANOTHER_SPECIAL = \"value_with_!_&_*\"\n");
    }

    #[test]
    fn test_print_list_as_variables_with_numeric_values() {
        let mut writer = Vec::new();
        let variables = vec![
            ("NUMERIC".to_string(), "12345".to_string()),
            ("FLOAT".to_string(), "67.89".to_string()),
        ];
        print_list_as_variables(&mut writer, variables);
        let result = String::from_utf8(writer).unwrap();
        assert_eq!(result, "NUMERIC = \"12345\"\nFLOAT = \"67.89\"\n");
    }

    #[test]
    fn test_print_list_as_variables_with_long_values() {
        let mut writer = Vec::new();
        let variables = vec![
            ("LONG_KEY".to_string(), "a".repeat(1000)),
            ("ANOTHER_LONG_KEY".to_string(), "b".repeat(2000)),
        ];
        print_list_as_variables(&mut writer, variables);
        let result = String::from_utf8(writer).unwrap();
        assert_eq!(result, format!("LONG_KEY = \"{}\"\nANOTHER_LONG_KEY = \"{}\"\n", "a".repeat(1000), "b".repeat(2000)));
    }
}
