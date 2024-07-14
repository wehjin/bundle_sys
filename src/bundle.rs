use std::any::{type_name, type_name_of_val};
use std::rc::Rc;

use anymap::CloneAny;

use crate::key::Key;

type CloneMap = anymap::hashbrown::Map<dyn CloneAny>;

#[derive(Clone)]
struct Property<K, V>(pub K, pub V)
where
	K: Key,
	V: Clone + 'static;

pub struct Bundle(CloneMap);
impl Bundle {
	pub fn empty() -> Self { Self(CloneMap::new()) }
	pub fn len(&self) -> usize { self.0.len() }

	pub fn get<K: Key, V: Clone + 'static>(&self) -> Option<&Rc<V>>
	{
		if let Some(Property(key, value)) = self.0.get::<Property<K, Rc<V>>>() {
			debug_assert_eq!(type_name_of_val(key), type_name::<K>());
			return Some(value);
		}
		None
	}
	pub fn assoc<K: Key, V: Clone + 'static>(&self, key: K, value: V) -> Self
	{
		let mut data = self.0.clone();
		data.insert(Property(key, Rc::new(value)));
		Self(data)
	}
	pub fn dissoc<K: Key, V: Clone + 'static>(&self) -> Self
	{
		let mut data = self.0.clone();
		data.remove::<Property<K, Rc<V>>>();
		Self(data)
	}
}

#[cfg(test)]
pub mod tests {
	use crate::bundle::Bundle;
	use crate::fixtures::{Hello, World};

	#[test]
	fn no_value_when_empty() {
		let bundle = Bundle::empty();
		assert_eq!(bundle.len(), 0);
		assert_eq!(bundle.get::<Hello, String>(), None)
	}

	#[test]
	fn some_value_after_assoc_no_value_after_dissoc() {
		let first = Bundle::empty()
			.assoc(Hello, "Bob".to_string())
			;
		assert_eq!(first.len(), 1);
		assert_eq!(first.get::<Hello, String>().unwrap().as_str(), "Bob");

		let second = first
			.dissoc::<Hello,String>()
			;
		assert_eq!(first.len(), 1);
		assert_eq!(first.get::<Hello, String>().unwrap().as_str(), "Bob");
		assert_eq!(second.len(), 0);
		assert_eq!(second.get::<Hello, String>(), None);
	}

	#[test]
	fn same_key_with_second_value_replaces_value_only_in_second_bundle() {
		let first = Bundle::empty()
			.assoc(Hello, "Bob".to_string())
			;
		let second = first
			.assoc(Hello, "Marley".to_string())
			;
		assert_eq!(first.len(), 1);
		assert_eq!(first.get::<Hello, String>().unwrap().as_ref(), "Bob");
		assert_eq!(second.len(), 1);
		assert_eq!(second.get::<Hello, String>().unwrap().as_ref(), "Marley");
	}

	#[test]
	fn second_key_adds_second_value_only_in_second_bundle() {
		let first = Bundle::empty()
			.assoc(Hello, "Jack".to_string())
			;
		let second = first
			.assoc(World, "Jill".to_string())
			;
		assert_eq!(first.len(), 1);
		assert_eq!(first.get::<Hello, String>().unwrap().as_ref(), "Jack");
		assert_eq!(first.get::<World, String>(), None);
		assert_eq!(second.len(), 2);
		assert_eq!(second.get::<Hello, String>().unwrap().as_ref(), "Jack");
		assert_eq!(second.get::<World, String>().unwrap().as_ref(), "Jill");
	}


	#[test]
	#[ignore]
	fn assoc_in_creates_sub_bundles() {
		// let hello_and_world = assoc_in(
		// 	&empty(),
		// 	[Hello.as_key(), World.as_key()],
		// 	JsString::from("Bob"),
		// );
		// let just_world = bundle::get::<Bundle>(&hello_and_world, &Hello).unwrap();
		// let bob_from_just_world = bundle::get::<JsString>(&just_world, &World);
		// let bob_from_hello_world = bundle::get_in::<JsString>(
		// 	&hello_and_world,
		// 	[Hello.as_key(), World.as_key()],
		// );
		// assert_eq!(bob_from_just_world, bob_from_hello_world);
	}
}
