pub trait IntoResponse {
    fn into_response(self) -> String;
}

impl IntoResponse for String {
    fn into_response(self) -> String {
        self
    }
}

impl IntoResponse for &'_ str {
    fn into_response(self) -> String {
        self.to_string()
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> String {
        match self {
            Ok(v) => v.into_response(),
            Err(e) => e.into_response(),
        }
    }
}
