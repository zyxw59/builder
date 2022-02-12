#[derive(Debug)]
pub enum Error {
    InvalidShape(&'static str, &'static str),
    UnexpectedIdent,
    MissingIdent,
    Multiple(Vec<Error>),
}
