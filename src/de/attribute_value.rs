use crate::{static_str_to_js, Error, ObjectExt};
use serde::de;
use wasm_bindgen::{JsCast, JsValue};

pub struct Deserializer {
    value: JsValue,
}

impl From<JsValue> for Deserializer {
    fn from(value: JsValue) -> Self {
        Self { value }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, _v: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unreachable!()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 u8 u16 u32 u64 u128 f32 f64 char str
        bytes byte_buf option unit unit_struct newtype_struct enum identifier
        tuple seq map tuple_struct ignored_any struct
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.value.is_object() {
            let obj = self.value.unchecked_into::<ObjectExt>();
            let js_field = static_str_to_js("N");
            let next_value = obj.get_with_ref_key(&js_field);
            // If this value is `undefined`, it might be actually a missing field;
            // double-check with an `in` operator if so.
            let is_missing_field = next_value.is_undefined() && !js_field.js_in(&obj);
            if is_missing_field {
                Err(Error::UnexpectedType("N"))
            } else if let Some(v) = next_value.as_string() {
                match v.parse::<i64>() {
                    Ok(v) => visitor.visit_i64(v),
                    Err(_) => Err(de::Error::custom(
                        "Couldn't deserialize i64 from a BigInt outside i64::MIN..i64::MAX bounds",
                    )),
                }
            } else {
                Err(Error::UnexpectedValue(next_value))
            }
        } else {
            Err(Error::UnsupportedType)
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.value.is_object() {
            let obj = self.value.unchecked_into::<ObjectExt>();
            let js_field = static_str_to_js("N");
            let next_value = obj.get_with_ref_key(&js_field);
            // If this value is `undefined`, it might be actually a missing field;
            // double-check with an `in` operator if so.
            let is_missing_field = next_value.is_undefined() && !js_field.js_in(&obj);
            if is_missing_field {
                Err(Error::UnexpectedType("N"))
            } else if let Some(v) = next_value.as_string() {
                match v.parse::<i128>() {
                    Ok(v) => visitor.visit_i128(v),
                    Err(_) => Err(de::Error::custom(
                        "Couldn't deserialize i128 from a BigInt outside i128::MIN..i128::MAX bounds",
                    )),
                }
            } else {
                Err(Error::UnexpectedValue(next_value))
            }
        } else {
            Err(Error::UnsupportedType)
        }
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if self.value.is_object() {
            let obj = self.value.unchecked_into::<ObjectExt>();
            let js_field = static_str_to_js("S");
            let next_value = obj.get_with_ref_key(&js_field);
            // If this value is `undefined`, it might be actually a missing field;
            // double-check with an `in` operator if so.
            let is_missing_field = next_value.is_undefined() && !js_field.js_in(&obj);
            if is_missing_field {
                Err(Error::UnexpectedType("S"))
            } else if let Some(v) = next_value.as_string() {
                visitor.visit_string(v)
            } else {
                Err(Error::UnexpectedValue(next_value))
            }
        } else {
            Err(Error::UnsupportedType)
        }
    }
}
