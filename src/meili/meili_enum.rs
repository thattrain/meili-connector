pub enum AuthError{
    MissingMasterKey
}
pub enum MeiliError{
    IndexPrimaryKeyMultipleCandidateFound,
    IndexPrimaryKeyNoCandidateFound,
    InvalidDocumentId,
    MissingDocumentId,
}

pub enum TaskStatus{
    Enqueued,
    Processing,
    Succeeded,
    Failed,
    Canceled
}

#[derive(Debug)]
pub enum Event {
    Insert,
    Update,
    Delete
}