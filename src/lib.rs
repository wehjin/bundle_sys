pub mod bundle {
	use std::any::type_name_of_val;

	use web_sys::js_sys::{Object, Reflect};
	use web_sys::wasm_bindgen::{JsCast, JsValue};

	pub trait Key {
		fn to_js(&self) -> JsValue { type_name_of_val(self).into() }

		fn as_key(&self) -> &dyn Key
		where
			Self: Sized,
		{ self }
	}
	impl<T: Key> Key for Box<T> {}

	pub trait TryGetBundleValue {
		fn try_get<V: JsCast>(&self, b: &Bundle) -> Option<V>;
	}

	impl<A: Key> TryGetBundleValue for A {
		fn try_get<V: JsCast>(&self, b: &Bundle) -> Option<V> { try_get(&b, self) }
	}

	pub type Bundle = Object;

	pub fn empty() -> Bundle { Object::new() }

	pub fn try_get<V: JsCast>(b: &Bundle, key: &dyn Key) -> Option<V>
	{
		match Reflect::get(b, &key.to_js()) {
			Ok(js_value) if js_value.is_undefined() => None,
			Ok(js_value) => Some(V::unchecked_from_js(js_value)),
			Err(_) => None
		}
	}
	pub fn try_get_in<V: JsCast>(b: &Bundle, keys: impl AsRef<[&'static dyn Key]>) -> Option<V>
	{
		let keys = keys.as_ref();
		match keys.len() {
			0 => None,
			1 => try_get::<V>(b, keys[0]),
			_ => {
				let mut b = b.clone();
				for i in 0..(keys.len() - 1) {
					match try_get::<Bundle>(&b, keys[i]) {
						None => return None,
						Some(child) => {
							b = child;
						}
					}
				}
				try_get::<V>(&b, keys[keys.len() - 1])
			}
		}
	}
	pub fn assoc(b: &Bundle, key: &dyn Key, value: impl JsCast) -> Bundle
	{
		let object = Object::new();
		Object::assign(&object, &b);
		Reflect::set(&object, &key.to_js(), &value.unchecked_into()).expect("reflect-set");
		object
	}
	pub fn assoc_in(b: &Bundle, keys: impl AsRef<[&'static dyn Key]>, value: impl JsCast) -> Bundle
	{
		let keys = keys.as_ref();
		match keys.len() {
			0 => copy(b),
			1 => assoc(b, keys[0], value),
			_ => {
				let value = match try_get::<Bundle>(b, keys[0]) {
					None => assoc_in(&empty(), &keys[1..], value),
					Some(existing) => assoc_in(&existing, &keys[1..], value),
				};
				assoc(b, keys[0], value)
			}
		}
	}
	pub fn dissoc(b: &Bundle, key: impl Key) -> Bundle
	{
		let object = copy(&b);
		Reflect::delete_property(&object, &key.to_js()).expect("reflect-delete");
		object
	}

	fn copy(b: &Bundle) -> Object
	{
		let object = Object::new();
		Object::assign(&object, &b);
		object
	}

	#[cfg(test)]
	pub mod tests {
		use wasm_bindgen_test::wasm_bindgen_test;
		use web_sys::js_sys::JsString;

		use crate::bundle;
		use crate::bundle::{assoc, assoc_in, Bundle, dissoc, empty, Key, TryGetBundleValue};

		#[derive(Copy, Clone)]
		pub struct HelloKey;
		impl Key for HelloKey {}

		#[derive(Copy, Clone)]
		pub struct WorldKey;
		impl Key for WorldKey {}

		#[wasm_bindgen_test]
		fn no_value_when_empty() {
			let bun = empty();
			assert_eq!(None, WorldKey {}.try_get::<JsString>(&bun));
		}

		#[wasm_bindgen_test]
		fn some_value_after_assoc() {
			let bun = assoc(&empty(), &WorldKey, JsString::from("Bob"));
			assert_eq!(Some(JsString::from("Bob")), WorldKey.try_get(&bun));
		}

		#[wasm_bindgen_test]
		fn no_value_after_dissoc() {
			let bun = assoc(&empty(), &WorldKey, JsString::from("Bob"));
			let bun = dissoc(&bun, WorldKey);
			assert_eq!(None, WorldKey.try_get::<JsString>(&bun));
		}

		#[wasm_bindgen_test]
		fn assoc_after_assoc_leaves_parent_untouched() {
			let a = assoc(&empty(), &WorldKey, JsString::from("Bob"));
			let b = assoc(&a, &WorldKey, JsString::from("Silent"));
			assert_ne!(WorldKey.try_get::<JsString>(&a), WorldKey.try_get::<JsString>(&b));
		}
		#[wasm_bindgen_test]
		fn assoc_in_creates_sub_bundles() {
			let hello_and_world = assoc_in(
				&empty(),
				[HelloKey.as_key(), WorldKey.as_key()],
				JsString::from("Bob"),
			);
			let just_world = bundle::try_get::<Bundle>(&hello_and_world, &HelloKey).unwrap();
			let bob_from_just_world = bundle::try_get::<JsString>(&just_world, &WorldKey);
			let bob_from_hello_world = bundle::try_get_in::<JsString>(
				&hello_and_world,
				[HelloKey.as_key(), WorldKey.as_key()],
			);
			assert_eq!(bob_from_just_world, bob_from_hello_world);
		}
	}
}