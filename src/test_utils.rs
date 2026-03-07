/// Enable logging within a unit test.
#[cfg(test)]
pub fn enable_logging() {
    // We don't want to check the Result, because the logger may
    // have already been initialized by another test
    let _ = env_logger::builder().is_test(true).try_init();
}
