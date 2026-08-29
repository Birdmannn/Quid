use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FactoryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    TemplateNotFound = 3,
    InvalidTemplate = 4,
    TemplateLimitReached = 5,
}
