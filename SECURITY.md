# Security Policy

If you discover a security vulnerability within QuorumScope, please use GitHub's private vulnerability reporting feature to disclose it confidentially to the maintainers. Do not open public issues for security vulnerabilities.

## Scope notes

- The API is read-only apart from `POST /api/v1/preflight`, which analyzes a transaction and does not submit it. Transaction XDR is not logged or stored, and request logs record only the method, path, status, and latency.
- Database errors are logged on the server and are not returned to clients.
- The preflight body is limited to 256 KiB.
- Report the affected endpoint or command, steps to reproduce, and the impact you observed. Do not include secrets.
