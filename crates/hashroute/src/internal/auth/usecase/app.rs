use crate::internal::auth::{repository::repo::{ Repo, UserRepo,DBError }, usecase::models::{RegisterInput, RegisterOutput}};

struct UseCase<T> where T: Repo {
    repo: T,
}

impl UseCase<UserRepo> {
    pub fn new(pool: sqlx::postgres::PgPool) -> Self {
        Self { repo: UserRepo::new(pool) }
    }

    pub fn Register(&self, user: RegisterInput) -> Result<RegisterOutput, DBError> {
        
    }
}


