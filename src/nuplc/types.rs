
#[derive(Debug)]
pub struct Unique(pub isize);
#[derive(Debug)]
pub struct DeBruijn(pub usize);

#[derive(Debug)]
pub struct Name {
    pub name: String,
    pub unique: Unique,
    pub debruijn: DeBruijn,
}
