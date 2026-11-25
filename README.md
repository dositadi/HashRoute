# HashRoute
HashRoute  is a lightweight, open-source URL shortening service designed to transform long, messy web addresses into clean, manageable links. Built with performance and simplicity in mind, the platform allows users to instantly compress URLs, redirect traffic efficiently, and manage their links through a minimalist interface.

# Minimal setup for PostgreSQL (swap out for mysql or sqlite if needed)
cargo install sqlx-cli --no-default-features --features native-tls,postgres

## Running

```sh
cp .env.example .env   # fill in JWT_SECRET and PG_PASSWORD
just cmp-up            # app on :8080 + Postgres; migrations run on startup
cargo test             # unit + router tests (no database needed)
```

For a browser frontend, list its origin in `ALLOW_ORIGINS` (e.g. `http://localhost:5173`,
with a scheme and no trailing slash), and set `PUBLIC_BASE_URL` to where short links are served.

## API reference

Base URL: `http://localhost:8080`. Request and response bodies are JSON.

### Response shape

Every response uses one of two envelopes:

```jsonc
// success
{ "data": { ... } }

// error
{
  "error": {
    "code": "validation_failed",          // stable, branch on this
    "message": "some fields are invalid", // human-readable
    "details": { "email": ["email is not valid"] } // validation errors only
  }
}
```

`204 No Content` responses have no body. Dates are RFC 3339 strings (`2026-11-25T14:00:00Z`).

### Auth

Protected routes need `Authorization: Bearer <access_token>`. When the API answers
`401` with code `token_expired`, call `rotate-refresh` and retry. Any other `401` means
log the user out.

| Method | Path | Body / headers | Success |
| --- | --- | --- | --- |
| POST | `/tiny/v1/auth/register` | `{ name, email, password }` | `201` `{ id, created_at }` |
| POST | `/tiny/v1/auth/login` | `{ email, password }` | `200` `Token` |
| POST | `/tiny/v1/auth/rotate-refresh` | header `X-Refresh: <refresh_token>` | `200` `Token` |
| GET | `/tiny/v1/auth/me` | bearer | `200` `Profile` |

Registration rules: name 3–20 characters; valid email; password at least 8 characters with a
lowercase letter, an uppercase letter and one of `$ # &`.

### Links (bearer required)

| Method | Path | Body | Success |
| --- | --- | --- | --- |
| POST | `/tiny/v1/links` | `{ "url": "https://..." }` | `201` `Link` |
| GET | `/tiny/v1/links` | – | `200` `Link[]`, newest first |
| GET | `/tiny/v1/links/{id}` | – | `200` `Link` |
| POST | `/tiny/v1/links/{id}/deactivate` | – | `204` |

Only `http` / `https` URLs with a host and no embedded credentials are accepted, up to 2048
characters. The stored URL is normalized (`https://example.com` → `https://example.com/`).

### Public

| Method | Path | Result |
| --- | --- | --- |
| GET | `/{code}` | `307` redirect to the original URL, or `404` JSON |
| GET | `/` | `200` `{ "data": { "status": "ok" } }` |

### Types (TypeScript)

```ts
type ApiSuccess<T> = { data: T };
type ApiError = {
  error: { code: ErrorCode; message: string; details?: Record<string, string[]> };
};

type Token = { access_token: string; refresh_token: string; expires_at: string };
type Profile = { id: string; name: string; email: string; created_at: string };
type Link = {
  id: string;
  short_code: string;
  short_url: string;          // PUBLIC_BASE_URL + "/" + short_code
  original_url: string;
  status: "active" | "expired";
  created_at: string;
  expires_at: string | null;
};

type ErrorCode =
  | "validation_failed" | "invalid_body" | "unsupported_media_type"
  | "invalid_credentials" | "email_taken"
  | "missing_token" | "invalid_token" | "token_expired"
  | "invalid_url" | "invalid_id" | "invalid_status"
  | "not_found" | "method_not_allowed"
  | "timeout" | "service_unavailable" | "internal_error";
```

### Error codes by status

| Status | Codes |
| --- | --- |
| 400 | `invalid_body` (malformed JSON), `invalid_id` |
| 401 | `invalid_credentials`, `missing_token`, `invalid_token`, `token_expired` |
| 404 | `not_found` |
| 405 | `method_not_allowed` |
| 409 | `email_taken` |
| 415 | `unsupported_media_type` |
| 422 | `validation_failed`, `invalid_url`, `invalid_status`, `invalid_body` (missing or wrongly typed field) |
| 500 | `internal_error` |
| 503 | `timeout`, `service_unavailable` |
