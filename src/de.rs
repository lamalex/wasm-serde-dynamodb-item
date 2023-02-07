use super::Error;
use crate::static_str_to_js;
use crate::ObjectExt;
use serde::de::{self, IntoDeserializer};
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};

mod attribute_value;

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

/// A newtype that allows using any [`JsValue`] as a [`serde::Deserializer`].
pub struct Deserializer {
    value: JsValue,
}

impl From<JsValue> for Deserializer {
    fn from(value: JsValue) -> Self {
        Self { value }
    }
}

// Ideally this would be implemented for `JsValue` instead, but we can't because
// of the orphan rule.
impl<'de> IntoDeserializer<'de, Error> for Deserializer {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, _v: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct enum identifier
        tuple seq map tuple_struct ignored_any
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.value.is_object() {
            visitor.visit_map(ObjectAccess::new(
                self.value.unchecked_into::<ObjectExt>(),
                fields,
            ))
        } else {
            Err(Error::UnsupportedType)
        }
    }
}

struct ObjectAccess {
    obj: ObjectExt,
    fields: std::slice::Iter<'static, &'static str>,
    next_value: Option<attribute_value::Deserializer>,
}

impl ObjectAccess {
    fn new(obj: ObjectExt, fields: &'static [&'static str]) -> Self {
        Self {
            obj,
            fields: fields.iter(),
            next_value: None,
        }
    }
}

impl<'de> de::MapAccess<'de> for ObjectAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        debug_assert!(self.next_value.is_none());

        for field in &mut self.fields {
            let js_field = static_str_to_js(field);
            let next_value = self.obj.get_with_ref_key(&js_field);
            // If this value is `undefined`, it might be actually a missing field;
            // double-check with an `in` operator if so.
            let is_missing_field = next_value.is_undefined() && !js_field.js_in(&self.obj);
            if !is_missing_field {
                self.next_value = Some(attribute_value::Deserializer::from(next_value));
                return Ok(Some(seed.deserialize(str_deserializer(field))?));
            }
        }

        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.next_value.take().unwrap_throw())
    }
}

fn str_deserializer(s: &str) -> de::value::StrDeserializer<Error> {
    de::IntoDeserializer::into_deserializer(s)
}
