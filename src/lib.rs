pub mod bundle {
	use std::any::type_name_of_val;

	use web_sys::js_sys::{Object, Reflect};
	use web_sys::wasm_bindgen::{JsCast, JsValue};

	pub trait Key: Copy {
		fn as_str(&self) -> &'static str { type_name_of_val(self) }
		fn to_js(&self) -> JsValue { self.as_str().into() }
		fn get<V: JsCast>(self, bundle: &Bundle) -> Option<V> { get(&bundle, self) }
	}


	pub type Bundle = Object;

	pub fn new() -> Bundle { Object::new() }

	pub fn get<V: JsCast>(b: &Bundle, key: impl Key) -> Option<V> {
		match Reflect::get(b, &key.to_js()) {
			Ok(js_value) if js_value.is_undefined() => None,
			Ok(js_value) => Some(V::unchecked_from_js(js_value)),
			Err(_) => None
		}
	}
	pub fn assoc(b: &Bundle, key: impl Key, value: impl JsCast) -> Bundle {
		let object = Object::new();
		Object::assign(&object, &b);
		Reflect::set(&object, &key.to_js(), &value.unchecked_into()).expect("reflect-set");
		object
	}
	pub fn dissoc(b: &Bundle, key: impl Key) -> Bundle {
		let object = Object::new();
		Object::assign(&object, &b);
		Reflect::delete_property(&object, &key.to_js()).expect("reflect-delete");
		object
	}

	#[cfg(test)]
	pub mod tests {
		use wasm_bindgen_test::wasm_bindgen_test;
		use web_sys::js_sys::JsString;

		use crate::bundle;
		use crate::bundle::{assoc, dissoc, Key};

		#[derive(Copy, Clone)]
		pub struct HelloKey;
		impl Key for HelloKey {}


		#[wasm_bindgen_test]
		fn no_value_when_empty() {
			let bun = bundle::new();
			assert_eq!(None, HelloKey.get::<JsString>(&bun));
		}

		#[wasm_bindgen_test]
		fn some_value_after_assoc() {
			let bun = bundle::new();
			let bun = assoc(&bun, HelloKey, JsString::from("Bob"));
			assert_eq!(Some(JsString::from("Bob")), HelloKey.get(&bun));
		}

		#[wasm_bindgen_test]
		fn no_value_after_dissoc() {
			let bun = bundle::new();
			let bun = assoc(&bun, HelloKey, JsString::from("Bob"));
			let bun = dissoc(&bun, HelloKey);
			assert_eq!(None, HelloKey.get::<JsString>(&bun));
		}
	}
}