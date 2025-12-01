use unified_shared::error::DomainError;
use uuid::Uuid;

pub fn parse_uuid(value: &str) -> Result<Uuid, DomainError> {
    Uuid::parse_str(value).map_err(|err| DomainError::Internal(err.to_string()))
}

