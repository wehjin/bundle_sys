pub mod bundle {
	use std::any::type_name_of_val;

	use web_sys::js_sys::{Object, Reflect};
	use web_sys::wasm_bindgen::{JsCast, JsValue};

	pub trait Key {
		fn to_js(&self) -> JsValue { type_name_of_val(self).into() }
	}
	impl<T: Key> Key for Box<T> {}

	pub trait Get {
		fn get<V: JsCast>(self, b: &Bundle) -> Option<V>;
	}

	impl<A: Key> Get for A {
		fn get<V: JsCast>(self, b: &Bundle) -> Option<V> { get(&b, self) }
	}

	#[derive(Clone)]
	struct ShadowKey(JsValue);
	impl Key for ShadowKey {
		fn to_js(&self) -> JsValue { self.0.clone() }
	}


	pub type Bundle = Object;

	pub fn empty() -> Bundle { Object::new() }

	pub fn get<V: JsCast>(b: &Bundle, key: impl Key) -> Option<V> {
		match Reflect::get(b, &key.to_js()) {
			Ok(js_value) if js_value.is_undefined() => None,
			Ok(js_value) => Some(V::unchecked_from_js(js_value)),
			Err(_) => None
		}
	}
	pub fn get_in<V: JsCast>(b: &Bundle, keys: &[Box<dyn Key>]) -> Option<V> {
		match keys.len() {
			0 => None,
			1 => get::<V>(b, ShadowKey(keys[0].to_js())),
			_ => {
				let mut b = b.clone();
				for i in 0..(keys.len() - 1) {
					match get::<Bundle>(&b, ShadowKey(keys[i].to_js())) {
						None => return None,
						Some(child) => {
							b = child;
						}
					}
				}
				get::<V>(&b, ShadowKey(keys[keys.len() - 1].to_js()))
			}
		}
	}
	pub fn assoc(b: &Bundle, key: impl Key, value: impl JsCast) -> Bundle {
		let object = Object::new();
		Object::assign(&object, &b);
		Reflect::set(&object, &key.to_js(), &value.unchecked_into()).expect("reflect-set");
		object
	}
	pub fn assoc_in(b: &Bundle, keys: &[Box<dyn Key>], value: impl JsCast) -> Bundle {
		match keys.len() {
			0 => copy(b),
			1 => assoc(b, ShadowKey(keys[0].to_js()), value),
			_ => {
				let value = match get::<Bundle>(b, ShadowKey(keys[0].to_js())) {
					None => assoc_in(&empty(), &keys[1..], value),
					Some(existing) => assoc_in(&existing, &keys[1..], value),
				};
				assoc(b, ShadowKey(keys[0].to_js()), value)
			}
		}
	}
	pub fn dissoc(b: &Bundle, key: impl Key) -> Bundle {
		let object = copy(&b);
		Reflect::delete_property(&object, &key.to_js()).expect("reflect-delete");
		object
	}

	fn copy(b: &Bundle) -> Object {
		let object = Object::new();
		Object::assign(&object, &b);
		object
	}

	#[cfg(test)]
	pub mod tests {
		use wasm_bindgen_test::wasm_bindgen_test;
		use web_sys::js_sys::JsString;

		use crate::bundle;
		use crate::bundle::{assoc, assoc_in, Bundle, dissoc, empty, Get, Key};

		#[derive(Copy, Clone)]
		pub struct HelloKey;
		impl Key for HelloKey {}

		#[derive(Copy, Clone)]
		pub struct WorldKey;
		impl Key for WorldKey {}

		#[wasm_bindgen_test]
		fn no_value_when_empty() {
			let bun = empty();
			assert_eq!(None, WorldKey {}.get::<JsString>(&bun));
		}

		#[wasm_bindgen_test]
		fn some_value_after_assoc() {
			let bun = assoc(&empty(), WorldKey, JsString::from("Bob"));
			assert_eq!(Some(JsString::from("Bob")), WorldKey.get(&bun));
		}

		#[wasm_bindgen_test]
		fn no_value_after_dissoc() {
			let bun = assoc(&empty(), WorldKey, JsString::from("Bob"));
			let bun = dissoc(&bun, WorldKey);
			assert_eq!(None, WorldKey.get::<JsString>(&bun));
		}

		#[wasm_bindgen_test]
		fn assoc_after_assoc_leaves_parent_untouched() {
			let a = assoc(&empty(), WorldKey, JsString::from("Bob"));
			let b = assoc(&a, WorldKey, JsString::from("Silent"));
			assert_ne!(WorldKey.get::<JsString>(&a), WorldKey.get::<JsString>(&b));
		}

		#[wasm_bindgen_test]
		fn assoc_in_creates_sub_bundles() {
			let parent = assoc_in(&empty(), &[Box::new(HelloKey), Box::new(WorldKey)], JsString::from("Bob"));
			let child = bundle::get::<Bundle>(&parent, HelloKey).unwrap();
			let from_child = bundle::get::<JsString>(&child, WorldKey);
			let from_parent = bundle::get_in::<JsString>(&parent, &[Box::new(HelloKey), Box::new(WorldKey)]);
			assert_eq!(from_child, from_parent);
		}
	}
}