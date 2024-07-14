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

	pub trait Get {
		fn get<V: JsCast>(&self, b: &Bundle) -> Option<V>;
	}

	impl<A: Key> Get for A {
		fn get<V: JsCast>(&self, b: &Bundle) -> Option<V> { get(&b, self) }
	}

	pub type Bundle = Object;

	pub fn empty() -> Bundle { Object::new() }

	pub fn get<V: JsCast>(bundle: &Bundle, key: &dyn Key) -> Option<V>
	{
		match Reflect::get(bundle, &key.to_js()) {
			Ok(js_value) if js_value.is_undefined() => None,
			Ok(js_value) => Some(V::unchecked_from_js(js_value)),
			Err(_) => None
		}
	}
	pub fn get_in<V: JsCast>(bundle: &Bundle, keys: impl AsRef<[&'static dyn Key]>) -> Option<V>
	{
		let keys = keys.as_ref();
		match keys.len() {
			0 => None,
			1 => get::<V>(bundle, keys[0]),
			_ => {
				let mut b = bundle.clone();
				for i in 0..(keys.len() - 1) {
					match get::<Bundle>(&b, keys[i]) {
						None => return None,
						Some(child) => {
							b = child;
						}
					}
				}
				get::<V>(&b, keys[keys.len() - 1])
			}
		}
	}
	pub fn assoc(bundle: &Bundle, key: &dyn Key, value: impl JsCast) -> Bundle
	{
		let object = Object::new();
		Object::assign(&object, &bundle);
		Reflect::set(&object, &key.to_js(), &value.unchecked_into()).expect("reflect-set");
		object
	}
	pub fn assoc_in(bundle: &Bundle, keys: impl AsRef<[&'static dyn Key]>, value: impl JsCast) -> Bundle
	{
		let keys = keys.as_ref();
		match keys.len() {
			0 => copy(bundle),
			1 => assoc(bundle, keys[0], value),
			_ => {
				let new_child = match get::<Bundle>(bundle, keys[0]) {
					None => assoc_in(&empty(), &keys[1..], value),
					Some(existing_child) => assoc_in(&existing_child, &keys[1..], value),
				};
				assoc(bundle, keys[0], new_child)
			}
		}
	}
	pub fn dissoc(bundle: &Bundle, key: impl Key) -> Bundle
	{
		let object = copy(&bundle);
		Reflect::delete_property(&object, &key.to_js()).expect("reflect-delete");
		object
	}

	fn copy(bundle: &Bundle) -> Bundle
	{
		let object = Object::new();
		Object::assign(&object, &bundle);
		object
	}

	#[cfg(test)]
	pub mod tests {
		use wasm_bindgen_test::wasm_bindgen_test;
		use web_sys::js_sys::JsString;

		use crate::bundle;
		use crate::bundle::{assoc, assoc_in, Bundle, dissoc, empty, Get, Key};

		#[derive(Copy, Clone)]
		pub struct Hello;
		impl Key for Hello {}

		#[derive(Copy, Clone)]
		pub struct World;
		impl Key for World {}

		#[wasm_bindgen_test]
		fn no_value_when_empty() {
			let bun = empty();
			assert_eq!(None, World.get::<JsString>(&bun));
		}

		#[wasm_bindgen_test]
		fn some_value_after_assoc() {
			let bun = assoc(&empty(), &World, JsString::from("Bob"));
			assert_eq!(Some(JsString::from("Bob")), World.get(&bun));
		}

		#[wasm_bindgen_test]
		fn no_value_after_dissoc() {
			let bun = assoc(&empty(), &World, JsString::from("Bob"));
			let bun = dissoc(&bun, World);
			assert_eq!(None, World.get::<JsString>(&bun));
		}

		#[wasm_bindgen_test]
		fn assoc_after_assoc_leaves_parent_untouched() {
			let a = assoc(&empty(), &World, JsString::from("Bob"));
			let b = assoc(&a, &World, JsString::from("Silent"));
			assert_ne!(World.get::<JsString>(&a), World.get::<JsString>(&b));
		}
		#[wasm_bindgen_test]
		fn assoc_in_creates_sub_bundles() {
			let hello_and_world = assoc_in(
				&empty(),
				[Hello.as_key(), World.as_key()],
				JsString::from("Bob"),
			);
			let just_world = bundle::get::<Bundle>(&hello_and_world, &Hello).unwrap();
			let bob_from_just_world = bundle::get::<JsString>(&just_world, &World);
			let bob_from_hello_world = bundle::get_in::<JsString>(
				&hello_and_world,
				[Hello.as_key(), World.as_key()],
			);
			assert_eq!(bob_from_just_world, bob_from_hello_world);
		}
	}
}