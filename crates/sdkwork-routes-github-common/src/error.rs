use sdkwork_github_integration_service::error::ServiceError;

use crate::response::ApiProblem;

pub fn map_service_error(error: ServiceError) -> ApiProblem {
    match error {
        ServiceError::Validation(message) => ApiProblem::bad_request(message),
        ServiceError::Configuration(message) => ApiProblem::unavailable(message),
        ServiceError::Integration(message) => ApiProblem::bad_gateway(message),
        ServiceError::Repository(message) => ApiProblem::internal_server_error(message),
        ServiceError::NotFound(message) => ApiProblem::not_found(message),
    }
}
