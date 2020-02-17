
pub trait ToSimpleError {
    fn to_simple_error(&self) -> simple_error::SimpleError;
}