use std::rc::Rc;

use crate::bundle::Bundle;

pub trait Key: Copy + Clone + PartialEq + 'static {
	fn get<'a, V: Clone + 'static>(&self, bundle: &'a Bundle) -> Option<&'a Rc<V>> {
		bundle.get::<Self, V>()
	}
}

#[cfg(test)]
mod tests {
	use crate::bundle::Bundle;
	use crate::fixtures::Hello;
	use crate::key::Key;

	#[test]
	fn retrieves_value_from_bundle() {
		let bundle = Bundle::empty()
			.assoc(Hello, "yo".to_string())
			;
		assert_eq!(Hello.get::<String>(&bundle).unwrap().as_str(), "yo");
	}
}
