use nutype::nutype;

#[nutype(derive(Debug,Clone,Copy,PartialEq,Eq,TryFrom,Into,Deserialize),validate(predicate = |i| *i > 0))]
pub struct NonZeroU31(i32);

impl NonZeroU31 {
    pub fn as_usize(self) -> usize {
        self.into_inner() as usize
    }
}
