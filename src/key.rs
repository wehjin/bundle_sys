use std::rc::Rc;

use crate::bundle::Bundle;

pub trait Key: Copy + Clone + PartialEq + 'static {
	type ValueType: Clone + 'static;
	fn get<'a>(&self, bundle: &'a Bundle) -> Option<&'a Rc<Self::ValueType>> {
		bundle.get::<Self>()
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
		assert_eq!(Hello.get(&bundle).unwrap().as_str(), "yo");
	}
}
