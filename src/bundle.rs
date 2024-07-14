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

#[derive(Debug, Clone)]
pub struct Bundle(HashMap<String, BundleValue>);
impl Bundle {
	pub fn empty() -> Self { Self(HashMap::new()) }
	pub fn len(&self) -> usize { self.0.len() }

	pub fn get<V: Clone + 'static>(&self, key: impl AsRef<str>) -> Option<Rc<V>>
	{
		self.0.get(key.as_ref())
			.map(|value| value.get_rc::<V>())
	}
	pub fn get_in<'a, V: Clone + 'static>(&self, keys: impl AsRef<[&'a str]> + Sized) -> Option<Rc<V>> {
		let keys = keys.as_ref();
		match keys.len() {
			0 => None,
			1 => self.get(keys[0]),
			_ => match self.get::<Bundle>(keys[0]) {
				Some(child) => child.get_in(&keys[1..]),
				None => None,
			},
		}
	}

	pub fn assoc<V: Clone + 'static>(&self, key: impl AsRef<str>, value: V) -> Self
	{
		let mut data = self.0.clone();
		data.insert(key.as_ref().to_string(), BundleValue::new(value));
		Self(data)
	}
	pub fn assoc_in<'a, V: Clone + 'static>(&self, keys: impl AsRef<[&'a str]> + Sized, value: V) -> Self {
		let keys = keys.as_ref();
		match keys.len() {
			0 => self.clone(),
			1 => self.assoc(keys[0], value),
			_ => {
				let old_child = self.get::<Bundle>(keys[0]).unwrap_or_else(|| Rc::new(Bundle::empty()));
				let new_child = old_child.assoc_in(&keys[1..], value);
				self.assoc(keys[0], new_child)
			}
		}
	}
	pub fn dissoc<V: Clone + 'static>(&self, key: impl AsRef<str>) -> Self
	{
		let mut data = self.0.clone();
		data.remove(key.as_ref());
		Self(data)
	}

	pub fn update<V: Clone + 'static, W: Clone + 'static>(
		&self,
		key: impl AsRef<str>,
		f: impl Fn(Option<Rc<V>>) -> W,
	) -> Self {
		let key = key.as_ref();
		let old = self.get::<V>(key);
		let new = f(old);
		self.assoc(key, new)
	}

	pub fn update_in<'a, V: Clone + 'static, W: Clone + 'static>(
		&self,
		keys: impl AsRef<[&'a str]> + Sized,
		f: impl Fn(Option<Rc<V>>) -> W,
	) -> Self {
		let keys = keys.as_ref();
		match keys.len() {
			0 => self.clone(),
			1 => self.update(keys[0], f),
			_ => {
				let child = self.get::<Bundle>(keys[0]).unwrap_or_else(|| Rc::new(Bundle::empty()));
				let new_child = child.update_in(&keys[1..], f);
				self.assoc(keys[0], new_child)
			}
		}
	}
}

#[cfg(test)]
pub mod tests {
	use std::rc::Rc;

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
	fn assoc_in_creates_sub_bundles() {
		let hello_and_world = Bundle::empty().assoc_in(["hello", "world"], "Bob".to_string());
		assert_eq!(hello_and_world.len(), 1);
		let just_world = hello_and_world.get::<Bundle>("hello").unwrap();
		assert_eq!(just_world.len(), 1);

		let bob_from_just_world = just_world.get::<String>("world");
		let bob_from_hello_world = hello_and_world.get_in::<String>(["hello", "world"]);
		assert_eq!(bob_from_just_world, bob_from_hello_world);
		assert_eq!(bob_from_just_world, Some(Rc::new("Bob".to_string())));
	}

	#[test]
	fn update() {
		let bundle = Bundle::empty()
			.assoc("hello", 33i32)
			.update(
				"hello",
				|old| {
					assert_eq!(old, Some(Rc::new(33i32)));
					34i32
				},
			);
		assert_eq!(bundle.get::<i32>("hello"), Some(Rc::new(34i32)));
	}

	#[test]
	fn update_in() {
		let bundle = Bundle::empty()
			.assoc_in(["hello", "hello", "world"], "world")
			.assoc_in(["hello", "hello", "universe"], "universe")
			.update_in(
				["hello", "hello"],
				|from: Option<Rc<Bundle>>| from.unwrap().dissoc::<&str>("world"),
			);
		assert_eq!(
			bundle.get_in::<&str>(["hello", "hello", "universe"]),
			Some(Rc::new("universe"))
		);
		assert_eq!(
			bundle.get_in::<&str>(["hello", "hello", "world"]),
			None
		);
	}
}
