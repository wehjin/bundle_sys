use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct BundleValue(Rc<dyn Any>);

impl BundleValue {
	pub fn new<V: Clone + 'static>(value: V) -> Self {
		Self(Rc::new(value))
	}
	pub fn get_rc<V: Clone + 'static>(&self) -> Rc<V> {
		self.0.clone().downcast::<V>().expect("downcast")
	}
}

#[derive(Debug)]
pub struct Bundle(HashMap<String, BundleValue>);
impl Bundle {
	pub fn empty() -> Self { Self(HashMap::new()) }
	pub fn len(&self) -> usize { self.0.len() }

	pub fn get<V: Clone + 'static>(&self, key: impl AsRef<str>) -> Option<Rc<V>>
	{
		self.0.get(key.as_ref())
			.map(|value| value.get_rc::<V>())
	}
	pub fn assoc<V: Clone + 'static>(&self, key: impl AsRef<str>, value: V) -> Self
	{
		let mut data = self.0.clone();
		data.insert(key.as_ref().to_string(), BundleValue::new(value));
		Self(data)
	}
	pub fn dissoc<V: Clone + 'static>(&self, key: impl AsRef<str>) -> Self
	{
		let mut data = self.0.clone();
		data.remove(key.as_ref());
		Self(data)
	}
}

#[cfg(test)]
pub mod tests {
	use crate::bundle::Bundle;

	#[test]
	fn no_value_when_empty() {
		let bundle = Bundle::empty();
		assert_eq!(bundle.len(), 0);
		assert_eq!(bundle.get::<String>("hello"), None)
	}

	#[test]
	fn stores_different_types() {
		let bundle = Bundle::empty()
			.assoc("hello", "Bob".to_string())
			.assoc("world", 32usize)
			;
		assert_eq!(bundle.len(), 2);
		assert_eq!(bundle.get::<String>("hello").unwrap().as_str(), "Bob");
		assert_eq!(bundle.get::<usize>("world").unwrap().as_ref(), &32usize);
	}

	#[test]
	fn some_value_after_assoc_and_no_value_after_dissoc() {
		let first = Bundle::empty()
			.assoc("hello", "Bob".to_string())
			;
		assert_eq!(first.len(), 1);
		assert_eq!(first.get::<String>("hello").unwrap().as_str(), "Bob");

		let second = first
			.dissoc::<String>("hello")
			;
		assert_eq!(first.len(), 1);
		assert_eq!(first.get::<String>("hello").unwrap().as_str(), "Bob");
		assert_eq!(second.len(), 0);
		assert_eq!(second.get::<String>("hello"), None);
	}

	#[test]
	fn same_key_with_second_value_replaces_value_only_in_second_bundle() {
		let first = Bundle::empty().assoc("hello", "Bob".to_string());
		let second = first.assoc("hello", "Marley".to_string());
		assert_eq!(first.len(), 1);
		assert_eq!(first.get::<String>("hello").unwrap().as_ref(), "Bob");
		assert_eq!(second.len(), 1);
		assert_eq!(second.get::<String>("hello").unwrap().as_ref(), "Marley");
	}

	#[test]
	fn second_key_adds_second_value_only_in_second_bundle() {
		let first = Bundle::empty()
			.assoc("hello", "Jack".to_string())
			;
		let second = first
			.assoc("world", "Jill".to_string())
			;
		assert_eq!(first.len(), 1);
		assert_eq!(first.get::<String>("hello").unwrap().as_ref(), "Jack");
		assert_eq!(first.get::<String>("world"), None);
		assert_eq!(second.len(), 2);
		assert_eq!(second.get::<String>("hello").unwrap().as_ref(), "Jack");
		assert_eq!(second.get::<String>("world").unwrap().as_ref(), "Jill");
	}


	#[test]
	#[ignore]
	fn assoc_in_creates_sub_bundles() {
		// let hello_and_world = assoc_in(
		// 	&empty(),
		// 	[HELLO.as_key(), WORLD.as_key()],
		// 	JsString::from("Bob"),
		// );
		// let just_world = bundle::get::<Bundle>(&hello_and_world, &HELLO).unwrap();
		// let bob_from_just_world = bundle::get::<JsString>(&just_world, &WORLD);
		// let bob_from_hello_world = bundle::get_in::<JsString>(
		// 	&hello_and_world,
		// 	[HELLO.as_key(), WORLD.as_key()],
		// );
		// assert_eq!(bob_from_just_world, bob_from_hello_world);
	}
}
