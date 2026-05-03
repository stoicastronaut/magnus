# Route Provider API Calls Through the Rust Backend

Magnus routes Provider API calls through the Rust backend instead of making direct HTTP calls from the React frontend. This adds backend implementation work, but keeps Provider API Keys out of the frontend, centralizes Provider Protocol handling, enables native proxy and networking support, and gives Diagnostics one backend boundary for provider failures.
