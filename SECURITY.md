# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Rescope, please report it responsibly.

**Do NOT open a public issue for security vulnerabilities.**

Instead, please use [GitHub's private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing/privately-reporting-a-security-vulnerability) to submit your report.

### What to Include

- A description of the vulnerability
- Steps to reproduce the issue
- The potential impact
- Any suggested fixes (optional)

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial assessment**: Within 1 week
- **Fix and disclosure**: We aim to release a fix within 2 weeks of confirmation

## Scope

Since Rescope is a **local development emulator**, its threat model is limited:

- It is designed to run on `localhost` only
- It does not handle real user credentials or production data
- Management API authentication is intentionally optional

However, we still take security seriously and will address any vulnerabilities that could impact developers using Rescope in their development workflows.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | ✅ Yes             |
