pub type Tag = String;

pub struct CountedTag {
    pub name: Tag,
    pub count: i32, // fixme should be u32
}

#[derive(Clone, Copy, Debug)]
pub enum Sort {
    NameAsc,
    CountAsc,
}
