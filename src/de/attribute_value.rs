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

impl Deserializer {
    fn deserialize_from_attribute_value<'de, F, V>(
        self,
        type_id: &'static str,
        visit_fn: F,
    ) -> Result<V, Error>
    where
        F: FnOnce(String) -> Result<V, Error>,
    {
        if self.value.is_object() {
            let obj = self.value.unchecked_into::<ObjectExt>();
            let js_field = static_str_to_js(type_id);
            let next_value = obj.get_with_ref_key(&js_field);
            // If this value is `undefined`, it might be actually a missing field;
            // double-check with an `in` operator if so.
            let is_missing_field = next_value.is_undefined() && !js_field.js_in(&obj);
            if is_missing_field {
                Err(Error::UnexpectedType(type_id))
            } else if let Some(v) = next_value.as_string() {
                visit_fn(v)
            } else {
                Err(Error::UnexpectedValue(next_value))
            }
        } else {
            Err(Error::UnsupportedType)
        }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, _v: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 u8 u16 u32 u64 u128 f32 f64 char str
        bytes byte_buf option unit unit_struct newtype_struct enum identifier
        tuple seq map tuple_struct ignored_any struct
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_from_attribute_value("N", |s| match s.parse::<i32>() {
            Ok(v) => visitor.visit_i32(v),
            Err(_) => Err(de::Error::custom(
                "Couldn't deserialize i32 from a BigInt outside i32::MIN..i32::MAX bounds",
            )),
        })
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_from_attribute_value("N", |s| match s.parse::<i64>() {
            Ok(v) => visitor.visit_i64(v),
            Err(_) => Err(de::Error::custom(
                "Couldn't deserialize i64 from a BigInt outside i64::MIN..i64::MAX bounds",
            )),
        })
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_from_attribute_value("N", |s| match s.parse::<i128>() {
            Ok(v) => visitor.visit_i128(v),
            Err(_) => Err(de::Error::custom(
                "Couldn't deserialize i128 from a BigInt outside i128::MIN..i128::MAX bounds",
            )),
        })
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_from_attribute_value("S", |s| visitor.visit_string(s))
    }
}
