use crate::internal::config::app_config::RetryConfig;

#[derive(Clone, Debug)]
#[derive(Default)]
pub struct AppState {
    pub retry: RetryConfig,
}
