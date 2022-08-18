//! Value is a basic unit of csv struct
//!
//! Value can be either number or text.

use crate::error::{DcsvError, DcsvResult};
use regex::Regex;
use std::{fmt::Display, str::FromStr};

/// Length of limiter's attributes
pub const LIMITER_ATTRIBUTE_LEN: usize = 4;

/// Basic component of virtual data
///
/// Value can be either number or text.
/// - "Number" is a signed interger (isize)
/// - Text is simply any data
///
/// Dcsv doesn't support float type because float can change the "original" source while
/// overriding. Since dcsv's goal is about safe manipulation of csv value, float is not appropriate.
#[derive(Clone, Eq, PartialEq, PartialOrd, Debug)]
pub enum Value {
    Number(isize),
    Text(String),
}

impl Value {
    /// Get a type of value
    ///
    /// This returns a new variable "ValueType"
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Number(_) => ValueType::Number,
            Self::Text(_) => ValueType::Text,
        }
    }
    /// Convert string into value with given type
    ///
    /// This can fail when a given source cannot bed converted to isize
    pub fn from_str(src: &str, value_type: ValueType) -> DcsvResult<Self> {
        Ok(match value_type {
            ValueType::Number => {
                // Empty value is evaluated to 0
                if src.is_empty() {
                    return Ok(Value::Number(0));
                }

                let src_number = src.parse::<isize>().map_err(|_| {
                    DcsvError::InvalidValueType(format!("\"{}\" is not a valid number", src))
                })?;
                Value::Number(src_number)
            }
            ValueType::Text => Value::Text(src.to_string()),
        })
    }

    /// Create empty value
    ///
    /// Default values for each types are
    /// - Number : 0
    /// - Text : ""
    pub fn empty(value_type: ValueType) -> Self {
        match value_type {
            ValueType::Number => Self::Number(0),
            ValueType::Text => Self::Text(String::new()),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::Number(num) => num.to_string(),
            Self::Text(txt) => txt.to_string(),
        };
        write!(f, "{}", out)
    }
}

// This struct should not expose value directly
// because some limiters are mutually exclusive.
/// Limiter that costraints which data that Value can hold
///
/// VaulueLimiter has four properties
/// - type ( Eitehr number or text )
/// - default value
/// - variants ( Range of values )
/// - pattern ( Regex pattern )
#[derive(Default, Clone, Debug)]
pub struct ValueLimiter {
    // Allowed variant
    value_type: ValueType,
    default: Option<Value>,
    variant: Option<Vec<Value>>,
    pattern: Option<Regex>, // -> This better be a regex
}

impl Display for ValueLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "type : {}", self.value_type)?;
        if let Some(var) = &self.variant {
            writeln!(f, "default value : {:?}", &var)?;
        }
        if let Some(var) = &self.variant {
            write!(f, "variants : {:?}", var)
        } else if let Some(var) = &self.pattern {
            write!(f, "pattern : {:?}", var)
        } else {
            write!(f, "")
        }
    }
}

impl ValueLimiter {
    /// Check if given value can be converted to the type of valuelimiter
    pub fn is_convertible(&self, value: &Value) -> Option<ValueType> {
        // TODO
        // Only when value type matches limiter's type
        match self.value_type {
            ValueType::Number => {
                if let Value::Text(text) = value {
                    // Empty value can be converted to 0 without hassle
                    if text.is_empty() {
                        return Some(ValueType::Number);
                    }

                    // String to Number
                    match text.parse::<isize>() {
                        Ok(_) => Some(ValueType::Number),
                        Err(_) => None,
                    }
                } else {
                    // Number to number
                    Some(ValueType::Number)
                }
            }
            ValueType::Text => Some(ValueType::Text),
        }
    }

    /// Check if value qualifies
    pub fn qualify(&self, value: &Value) -> bool {
        if value.get_type() != self.get_type() {
            return false;
        }
        match value {
            Value::Number(num) => {
                if let Some(variant) = self.variant.as_ref() {
                    variant.contains(value)
                } else if let Some(pattern) = self.pattern.as_ref() {
                    pattern.is_match(&num.to_string())
                } else {
                    true
                }
            }
            Value::Text(text) => {
                if let Some(variant) = self.variant.as_ref() {
                    variant.contains(value)
                } else if let Some(pattern) = self.pattern.as_ref() {
                    pattern.is_match(text)
                } else {
                    true
                }
            }
        }
    }

    /// Create value limiter from attributes
    ///
    /// The order is
    /// - Type
    /// - Default
    /// - Variant
    /// - Pattern
    pub fn from_line(attributes: &[impl AsRef<str>]) -> DcsvResult<Self> {
        let attributes: Vec<&str> = attributes.iter().map(|s| s.as_ref()).collect();
        if attributes.len() != LIMITER_ATTRIBUTE_LEN {
            return Err(DcsvError::InvalidRowData(format!(
                "Schema row has insufficient columns \n= {:?}",
                attributes
            )));
        }
        let mut limiter = Self::default();
        let vt = ValueType::from_str(attributes[0])?;
        let default = attributes[1];
        let variants = attributes[2];
        let pattern = attributes[3];
        limiter.set_type(vt);

        // Default value is necessary for complicated limiter
        if !default.is_empty() {
            let default = Value::from_str(default, vt)?;

            // DO variants
            if !variants.is_empty() {
                let mut values = vec![];
                for var in variants.split_whitespace() {
                    values.push(Value::from_str(var, vt)?);
                }
                limiter.set_variant(default, &values)?;
            } else if !pattern.is_empty() {
                // Do patterns
                limiter.set_pattern(
                    default,
                    Regex::new(pattern).expect("Failed to create pattern"),
                )?;
            } else {
                limiter.default = Some(default);
            }
        } else {
            // Default is empty
            if !pattern.is_empty() || !variants.is_empty() {
                return Err(DcsvError::InvalidLimiter(
                    "Either pattern or variants needs default value to be valid".to_string(),
                ));
            }
        }
        Ok(limiter)
    }

    /// Get type
    pub fn get_type(&self) -> ValueType {
        self.value_type
    }

    /// Set type
    pub fn set_type(&mut self, column_type: ValueType) {
        self.value_type = column_type;
    }

    /// Get default value from limiter
    pub fn get_default(&self) -> Option<&Value> {
        self.default.as_ref()
    }

    /// Return variant reference
    pub fn get_variant(&self) -> Option<&Vec<Value>> {
        self.variant.as_ref()
    }

    /// Set variant
    pub fn set_variant(&mut self, default: Value, variants: &[Value]) -> DcsvResult<()> {
        if !variants.contains(&default) {
            return Err(DcsvError::InvalidLimiter(
                "Default value should be among one of variants".to_string(),
            ));
        }
        self.default.replace(default);
        self.variant.replace(variants.to_vec());
        Ok(())
    }

    /// Get pattern
    pub fn get_pattern(&self) -> Option<&Regex> {
        self.pattern.as_ref()
    }

    /// Set pattern
    pub fn set_pattern(&mut self, default: Value, pattern: Regex) -> DcsvResult<()> {
        if !pattern.is_match(&default.to_string()) {
            return Err(DcsvError::InvalidLimiter(
                "Default value should match pattern".to_string(),
            ));
        }
        self.default.replace(default);
        self.pattern.replace(pattern);
        Ok(())
    }
}

/// Type of a value
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ValueType {
    Number,
    Text,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Number => "Number",
                Self::Text => "Text",
            }
        )
    }
}

impl std::str::FromStr for ValueType {
    type Err = DcsvError;

    /// This actually never fails
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "number" => Ok(Self::Number),
            "text" => Ok(Self::Text),
            _ => Err(DcsvError::InvalidValueType(
                "Value type should be either number or text".to_string(),
            )),
        }
    }
}

impl Default for ValueType {
    fn default() -> Self {
        Self::Text
    }
}
