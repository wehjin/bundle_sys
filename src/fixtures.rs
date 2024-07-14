use crate::key::Key;

#[derive(Copy, Clone, PartialEq)]
pub struct Hello;
impl Key for Hello {}

#[derive(Copy, Clone, PartialEq)]
pub struct World;
impl Key for World {}
