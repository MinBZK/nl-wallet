use nutype::nutype;

#[nutype(derive(Debug,Clone,Copy,TryFrom,Into,Deserialize),validate(predicate = |i| *i > 0))]
pub struct NonZeroU31(i32);
