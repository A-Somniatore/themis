# Users Service Contract v1

This directory contains the OpenAPI 3.1 contract for the Users Service v1.

## Overview

The Users Service provides user management capabilities:

- User registration (create)
- User retrieval (get, list)
- User updates (patch)
- User deletion (delete)

## Operations

| Operation    | Method | Path            | Description                    |
| ------------ | ------ | --------------- | ------------------------------ |
| `listUsers`  | GET    | /users          | List all users with pagination |
| `createUser` | POST   | /users          | Create a new user              |
| `getUser`    | GET    | /users/{userId} | Get user by ID                 |
| `updateUser` | PATCH  | /users/{userId} | Update user                    |
| `deleteUser` | DELETE | /users/{userId} | Delete user                    |

## Security

All operations require authentication:

- **spiffe**: Service-to-service mTLS (internal)
- **bearer**: JWT bearer token (external users)

## Changelog

### v1.0.0 (2026-01-04)

- Initial release
- Basic CRUD operations for users
- Pagination support for list endpoint
- Search by name/email
