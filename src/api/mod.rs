//! HTTP API layer compatible with KOReader sync protocol.
//!
//! This module implements a RESTful API that is compatible with the KOReader's synchronization
//! feature. It provides endpoints for user management and reading progress synchronization across
//! devices.
//!
//! # API Endpoints
//!
//! ## Public Endpoints (No Authentication Required)
//!
//! - `POST /users/create` - User registration
//! - `GET /robots.txt` - Robots exclusion file
//!
//! ## Authenticated Endpoints (Require x-auth-user and x-auth-key Headers)
//!
//! - `GET /users/auth` - User authentication and profile
//! - `PUT /syncs/progress` - Update reading progress
//! - `GET /syncs/progress/{document}` - Retrieve reading progress for a document
//! - `GET /healthcheck` - Health check endpoint
//!
//! # Authentication
//!
//! Authenticated endpoints use HTTP Basic Authentication with custom headers:
//! - `x-auth-user`: Username
//! - `x-auth-key`: Password
//!
//! The authentication middleware validates credentials and attaches user information
//! to the request context for use by route handlers.
//!
//! # Middleware
//!
//! The API includes several middleware layers:
//! - **Rate Limiting**: Prevents abuse by limiting requests per IP
//! - **Authentication**: Validates credentials for protected routes
//! - **Tracing**: Logs HTTP requests and responses
//! - **Error Handling**: Converts errors to appropriate HTTP responses
//!
//! - [`routes`] - HTTP route handlers for different endpoints
//! - [`middleware`] - Authentication, rate limiting, and other middleware
//! - [`router`] - Application router configuration
//! - [`state`] - Shared application state (database connection, etc.)
//! - [`error`] - API-specific error types and HTTP error responses
//!
pub mod error;
pub mod middleware;
pub mod router;
pub mod routes;
pub mod state;
