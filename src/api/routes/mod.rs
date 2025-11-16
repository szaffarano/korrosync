//! HTTP route handlers for the KOReader sync API.
//!
//! This module contains all the route handlers that implement the KOReader
//! synchronization protocol. Routes are organized by functionality and
//! grouped into public (unauthenticated) and protected (authenticated) endpoints.
//!
//! # Route Modules
//!
//! ## Public Routes (No Authentication Required)
//
//! - **[`register`]** - `POST /users/create`
//!   - User registration endpoint
//!
//! - **[`robots`]** - `GET /robots.txt`
//!   - Robots exclusion protocol file
//!   - Instructs web crawlers not to index the API
//!
//! - **[`fallback`]** - All unmatched routes
//!   - Returns 404 Not Found for invalid endpoints
//!
//! ## Protected Routes (Authentication Required)
//!
//! These routes require `x-auth-user` and `x-auth-key` headers for authentication.
//!
//! - **[`users_auth`]** - `GET /users/auth`
//!   - User authentication retrieval
//!   - Returns user information and last activity timestamp
//!
//! - **[`syncs_progress`]** - Progress synchronization endpoints
//!   - `PUT /syncs/progress` - Update reading progress for a document
//!   - `GET /syncs/progress/{document}` - Retrieve progress for a specific document
//!
//! - **[`healthcheck`]** - `GET /healthcheck`
//!   - Simple health check endpoint for monitoring
//!
//! # KOReader Compatibility
//!
//! These endpoints implement the KOReader sync protocol, ensuring compatibility with the
//! KOReader's synchronization plugin. The API follows REST principles and uses JSON for
//! request/response payloads.

pub mod fallback;
pub mod healthcheck;
pub mod register;
pub mod robots;
pub mod syncs_progress;
pub mod users_auth;
