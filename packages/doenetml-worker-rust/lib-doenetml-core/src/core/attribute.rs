use std::num::{ParseFloatError, ParseIntError};

use serde::{Deserialize, Serialize};

use crate::{
    components::prelude::DastAttribute, dast::DastTextMacroContent,
    state_var_interfaces::util::string_to_boolean,
};

/// camelCase
pub type AttributeName = &'static str;

#[derive(Debug)]
pub enum AttributeType {
    AttributeChildren,
    Reference,
}

pub mod attribute_type {
    use std::string::String as StdString;
    pub type Number = f64;
    pub type Integer = i64;
    pub type Boolean = bool;
    pub type String = StdString;
}

/// An attribute value can simple, e.g. a fixed string or number, or complex, e.g.
/// coming from a a macro or formula. This structure represents both. The `try_from`
/// method will attempt to convert to `PrimitiveType` if possible and return a `Complex(...)`
/// otherwise.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum AttributeValue<PrimitiveType> {
    /// A simple value, e.g. a fixed string or number
    Primitive(PrimitiveType),
    /// A complex value, e.g. coming from a a macro or formula
    Complex(Vec<DastTextMacroContent>),
}

impl<T: TryFromString> TryFrom<&DastAttribute> for AttributeValue<T> {
    type Error = T::Error;
    fn try_from(value: &DastAttribute) -> Result<Self, Self::Error> {
        if value.children.len() != 1 {
            return Ok(Self::Complex(value.children.clone()));
        }
        let attr = &value.children[0];
        if let DastTextMacroContent::Text(text) = attr {
            return Ok(Self::Primitive(T::try_from_string(&text.value)?));
        }
        Ok(Self::Complex(value.children.clone()))
    }
}

/// We cannot implement `TryFrom` on `String` for other builtin types, so we
/// create our own trait that can be implemented.
pub trait TryFromString
where
    Self: Sized,
{
    type Error;
    fn try_from_string(value: &str) -> Result<Self, Self::Error>;
}

/// We cannot implement `TryInto` to convert between `String` and other builtin types,
/// so we create our own trait that can be implemented.
pub trait TryIntoFromString<T>
where
    Self: Sized,
{
    type Error;
    /// Convert from a `String` to `T`. We cannot use `TryInto` because this needs
    /// to work with builtin types.
    fn try_into_from_string(self) -> Result<T, Self::Error>;
}

/// Generic implementation of `TryIntoFromString` that gets created for any type with the `TryFromString`
/// trait.
impl<T> TryIntoFromString<T> for &String
where
    T: TryFromString,
{
    type Error = T::Error;
    fn try_into_from_string(self) -> Result<T, Self::Error> {
        T::try_from_string(self)
    }
}

///////////////////////////////////////////////////////////////
// Implementations of `TryFromString` for various types
///////////////////////////////////////////////////////////////

impl TryFromString for attribute_type::Integer {
    type Error = ParseIntError;
    fn try_from_string(value: &str) -> Result<Self, Self::Error> {
        value.parse::<i64>()
    }
}

impl TryFromString for attribute_type::Number {
    type Error = ParseFloatError;
    fn try_from_string(value: &str) -> Result<Self, Self::Error> {
        value.parse::<f64>()
    }
}

impl TryFromString for attribute_type::Boolean {
    type Error = ();
    /// Conversion from a string to a bool can never fail.
    fn try_from_string(value: &str) -> Result<Self, Self::Error> {
        Ok(string_to_boolean(value))
    }
}

impl TryFromString for attribute_type::String {
    type Error = ();
    /// Conversion from a string to a string can never fail.
    fn try_from_string(value: &str) -> Result<Self, Self::Error> {
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_dast_attribute() {
        // An i64 should convert without error
        let dast_attribute: DastAttribute = serde_json::from_str(
            r#"{
            "type": "attribute",
            "name": "foo",
            "children": [
              {"type": "text", "value": "123"}
            ]
          }"#,
        )
        .unwrap();
        let val: Result<AttributeValue<i64>, _> = (&dast_attribute).try_into();
        assert_eq!(Ok(AttributeValue::Primitive(123)), val);

        // A float will fail to convert to an i64
        let dast_attribute: DastAttribute = serde_json::from_str(
            r#"{
            "type": "attribute",
            "name": "foo",
            "children": [
              {"type": "text", "value": "123.5"}
            ]
          }"#,
        )
        .unwrap();
        let val: Result<AttributeValue<i64>, _> = (&dast_attribute).try_into();
        assert!(val.is_err());

        // A complex type is passed through
        let dast_attribute: DastAttribute = serde_json::from_str(
            r#"{
            "type": "attribute",
            "name": "foo",
            "children": [
              {"type": "text", "value": "123"},
              {"type": "text", "value": "123"}
            ]
          }"#,
        )
        .unwrap();
        let val: Result<AttributeValue<i64>, _> = (&dast_attribute).try_into();
        assert!(matches!(val, Ok(AttributeValue::Complex(_))));
    }
}
