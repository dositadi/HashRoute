
pub mod handlers;
pub mod models;
/* 
#[derive(Debug, Clone, Default)]
pub struct ValidateForm<T>(pub T);

impl<S, T> FromRequest<S>
    for ValidateForm<T>
    where
        S: Send + Sync,
        T: DeserializeOwned + Validate,
        Form<T>: FromRequest<S, Rejection = FormRejection>
{
    type Rejection = AuthError;

    async fn from_request(
        req: axum::extract::Request,
        state: &S
    ) -> Result<Self, Self::Rejection> {
        let 
    }
}
 */