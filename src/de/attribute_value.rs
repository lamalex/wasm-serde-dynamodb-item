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

macro_rules! deserialize_from_numeric_attribute_value {
    ($($func:ident)*) => {
        $(deserialize_from_numeric_attribute_helper!{$func})*
    };
}

macro_rules! deserialize_from_numeric_attribute_helper {
    (f64) => {
        deserialize_from_float_numeric_attribute_method! {deserialize_f64<'de, V>()}
    };
    (f32) => {
        deserialize_from_float_numeric_attribute_method! {deserialize_f32<'de, V>()}
    };
    (i64) => {
        deserialize_from_signed_numeric_attribute_method! {deserialize_i64<'de, V>()}
    };
    (i32) => {
        deserialize_from_signed_numeric_attribute_method! {deserialize_i32<'de, V>()}
    };
    (i16) => {
        deserialize_from_signed_numeric_attribute_method! {deserialize_i16<'de, V>()}
    };
    (i8) => {
        deserialize_from_signed_numeric_attribute_method! {deserialize_i8<'de, V>()}
    };
    (u64) => {
        deserialize_from_unsigned_numeric_attribute_method! {deserialize_u64<'de, V>()}
    };
    (u32) => {
        deserialize_from_unsigned_numeric_attribute_method! {deserialize_u32<'de, V>()}
    };
    (u16) => {
        deserialize_from_unsigned_numeric_attribute_method! {deserialize_u16<'de, V>()}
    };
    (u8) => {
        deserialize_from_unsigned_numeric_attribute_method! {deserialize_u8<'de, V>()}
    };
}

macro_rules! deserialize_from_signed_numeric_attribute_method {
    ($func:ident<$l:tt, $v:ident>($($arg:ident : $ty:ty),*)) => {
        #[inline]
        fn $func<$v>(self, $($arg: $ty,)* visitor: $v) -> Result<$v::Value, Self::Error>
        where
            $v: de::Visitor<$l>,
        {
            $(
                let _ = $arg;
            )*
            self.deserialize_from_attribute_value("N", |s| match s.parse::<i64>() {
                Ok(v) => visitor.visit_i64(v),
                Err(_) => Err(de::Error::custom("Something went wrong deserializing into numeric"))
            })
        }
    };
}

macro_rules! deserialize_from_unsigned_numeric_attribute_method {
    ($func:ident<$l:tt, $v:ident>($($arg:ident : $ty:ty),*)) => {
        #[inline]
        fn $func<$v>(self, $($arg: $ty,)* visitor: $v) -> Result<$v::Value, Self::Error>
        where
            $v: de::Visitor<$l>,
        {
            $(
                let _ = $arg;
            )*
            self.deserialize_from_attribute_value("N", |s| match s.parse::<u64>() {
                Ok(v) => visitor.visit_u64(v),
                Err(_) => Err(de::Error::custom("Something went wrong deserializing into numeric"))
            })
        }
    };
}

macro_rules! deserialize_from_float_numeric_attribute_method {
    ($func:ident<$l:tt, $v:ident>($($arg:ident : $ty:ty),*)) => {
        #[inline]
        fn $func<$v>(self, $($arg: $ty,)* visitor: $v) -> Result<$v::Value, Self::Error>
        where
            $v: de::Visitor<$l>,
        {
            $(
                let _ = $arg;
            )*
            self.deserialize_from_attribute_value("N", |s| match s.parse::<f64>() {
                Ok(v) => visitor.visit_f64(v),
                Err(_) => Err(de::Error::custom("Something went wrong deserializing into numeric"))
            })
        }
    };
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
        bool char str
        bytes byte_buf option unit unit_struct newtype_struct enum identifier
        tuple seq map tuple_struct ignored_any struct
    }

    deserialize_from_numeric_attribute_value! { u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_from_attribute_value("N", |s| match s.parse::<u128>() {
            Ok(v) => visitor.visit_u128(v),
            Err(_) => Err(de::Error::custom(
                "Couldn't deserialize u128 from a BigInt outside u128::MIN..u128::MAX bounds",
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
