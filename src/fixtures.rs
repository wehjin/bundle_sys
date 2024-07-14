use crate::key::Key;

#[derive(Copy, Clone, PartialEq)]
pub struct Hello;
impl Key for Hello { type ValueType = String; }

#[derive(Copy, Clone, PartialEq)]
pub struct World;
impl Key for World { type ValueType = String; }
