// The FFI layer translates; it owns nothing the engine needs.
mod ffi {
    use crate::worker::{Worker, WorkerError};

    pub struct ApiError;

    impl From<WorkerError> for ApiError {
        fn from(_: WorkerError) -> Self {
            ApiError
        }
    }

    pub struct Client {
        worker: Worker,
    }
}

// The engine speaks its own errors and knows nothing about who calls it.
mod worker {
    pub struct Worker;

    pub struct WorkerError;

    impl Worker {
        pub fn run(&self) -> Result<(), WorkerError> {
            Ok(())
        }
    }
}
