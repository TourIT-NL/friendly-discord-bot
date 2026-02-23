# Discord Purge Logging Strategy: Ensuring Transparency and Debugging

This document outlines the robust **logging strategy** for the **Discord Purge utility**. Effective logging is paramount for monitoring application health, tracing user actions (locally), debugging issues, and ensuring transparency for this **Discord message deletion and privacy management tool**.

- **Library**: The `tracing` crate, a powerful framework for instrumenting Rust programs, will be exclusively used in the **Discord Purge backend** for structured logging. This provides detailed and context-rich log data.
- **Levels**: Log levels are judiciously applied to categorize events:
  - `INFO` for major lifecycle events (e.g., application start, successful user login to the **Discord cleanup tool**, completion of a **bulk message deletion** operation).
  - `WARN` for non-critical issues (e.g., a single API call to Discord fails but can be successfully retried by the **rate limiter**).
  - `ERROR` for critical failures that impact the functionality of the **Discord Purge application** and require immediate attention.
- **Output**: Log output is designed for both development and user-level insights:
  - During development, logs are printed to the console for real-time debugging of the **desktop application**.
  - For deployed versions, logs will be written to a rotating log file, stored securely in the user's application data directory (e.g., `.../app.log`). The `tracing_appender` crate will manage log file rotation, maintaining a maximum of 3 log files, each capped at 5MB, to prevent excessive disk usage while retaining sufficient history for troubleshooting **Discord privacy tool** operations.
