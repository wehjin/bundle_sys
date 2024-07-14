use std::any::type_name_of_val;

use web_sys::js_sys::{JsString, Object, Reflect};
use web_sys::wasm_bindgen::{JsCast, JsValue};

pub trait BunKey: Copy {
	type ValueType: JsCast;
	fn as_str(&self) -> &'static str { type_name_of_val(self) }
	fn to_js(&self) -> JsValue { self.as_str().into() }
	fn get(self, bun: &DataBun) -> Option<Self::ValueType> { bun.get(self) }
}

#[derive(Copy, Clone)]
pub struct HelloKey;
impl BunKey for HelloKey { type ValueType = JsString; }

#[derive(Debug)]
pub struct DataBun(Object);
impl DataBun {
	pub fn new() -> Self {
		Self(Object::new())
	}
	pub fn get<K, V>(&self, key: K) -> Option<V>
	where
		K: BunKey<ValueType=V>,
		V: JsCast,
	{
		match Reflect::get(&self.0, &key.to_js()) {
			Ok(js_value) if js_value.is_undefined() => None,
			Ok(js_value) => Some(V::unchecked_from_js(js_value)),
			Err(_) => None
		}
	}
	pub fn assoc<K, V>(&self, key: K, value: K::ValueType) -> Self
	where
		K: BunKey<ValueType=V>,
		V: JsCast,
	{
		let object = Object::new();
		Object::assign(&object, &self.0);
		Reflect::set(&object, &key.to_js(), &value.unchecked_into()).expect("reflect-set");
		Self(object)
	}
	pub fn dissoc<K, V>(&self, key: K) -> Self
	where
		K: BunKey<ValueType=V>,
		V: JsCast,
	{
		let object = Object::new();
		Object::assign(&object, &self.0);
		Reflect::delete_property(&object, &key.to_js()).expect("reflect-delete");
		Self(object)
	}
}

#[cfg(test)]
pub mod tests {
	use wasm_bindgen_test::wasm_bindgen_test;

	use super::*;

	#[wasm_bindgen_test]
	fn no_value_when_empty() {
		let bun = DataBun::new();
		assert_eq!(None, HelloKey.get(&bun));
	}

	#[wasm_bindgen_test]
	fn some_value_after_assoc() {
		let bun = DataBun::new()
			.assoc(HelloKey, JsString::from("Bob"))
			;
		assert_eq!(Some(JsString::from("Bob")), HelloKey.get(&bun));
	}

	#[wasm_bindgen_test]
	fn no_value_after_dissoc() {
		let bun = DataBun::new()
			.assoc(HelloKey, JsString::from("Bob"))
			.dissoc(HelloKey)
			;
		assert_eq!(None, HelloKey.get(&bun));
	}
}
