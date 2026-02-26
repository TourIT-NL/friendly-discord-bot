// src-tauri/src/tests/error_test.rs

#[cfg(test)]
mod tests {
    use crate::core::error::AppError;
    use std::io::{Error, ErrorKind};

    #[test]
    fn test_io_error_conversion() {
        let io_err = Error::new(ErrorKind::NotFound, "test error");
        let app_err: AppError = io_err.into();
        assert_eq!(app_err.error_code, "io_error");
        assert!(app_err.technical_details.unwrap().contains("test error"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = "{ invalid: json }";
        let result: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(json_str);
        let json_err = result.unwrap_err();
        let app_err: AppError = json_err.into();
        assert_eq!(app_err.error_code, "json_error");
    }
}
