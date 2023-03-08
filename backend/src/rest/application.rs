use rest_api::application::{GetApplicationResponse, GetApplicationResult};

pub async fn get_application() -> GetApplicationResult {
    GetApplicationResult::Success(GetApplicationResponse {
        commit: env!("VERGEN_GIT_SHA_SHORT").to_string(),
        build_date: env!("SOURCE_TIMESTAMP").to_string(),
    })
}
