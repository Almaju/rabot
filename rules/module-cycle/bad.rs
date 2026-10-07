// The FFI layer: what the crate exposes to Kotlin and Swift.
mod ffi {
    use crate::worker::Worker;

    pub type ApiResult<T> = Result<T, ApiError>;

    pub struct ApiError;

    pub struct Client {
        worker: Worker,
    }
}

// The engine the FFI layer drives, reaching back up for its error type.
mod worker {
    use crate::ffi::ApiResult;

    pub struct Worker;

    impl Worker {
        pub fn run(&self) -> ApiResult<()> {
            Ok(())
        }
    }
}
