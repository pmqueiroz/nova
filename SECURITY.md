# Security Policy

## Supported Versions

Security fixes are applied to the latest release only. Older versions do not receive backports.

| Version | Supported |
|---|---|
| Latest release | Yes |
| Older releases | No |

## Reporting a Vulnerability

If you believe you've found a security vulnerability in Nova, please follow responsible disclosure practices and **do not open a public GitHub issue**. Disclosing a vulnerability publicly before a fix is available puts users at risk.

Instead, report it through one of the following private channels:

- **GitHub Security Advisory:** [Open a private advisory](https://github.com/pmqueiroz/nova/security/advisories/new) — preferred, as it allows coordinated disclosure directly on GitHub.
- **Email:** Reach the maintainer privately through the contact information on [the GitHub profile](https://github.com/pmqueiroz).

Please include:

- A description of the vulnerability and its potential impact.
- Steps to reproduce or a proof-of-concept.
- The Nova version and OS where you observed the issue.

You will receive an acknowledgment promptly. The maintainer will work with you to understand the issue and coordinate a fix and public disclosure.

## Scope

Nova is a local terminal emulator with no network-facing server component. Relevant security areas include:

- **PTY escape sequences** — malicious output that could break out of the terminal sandbox or execute unintended commands.
- **Config file handling** — arbitrary file reads or writes triggered by a crafted config.
- **AI integration** — leakage of terminal content or API keys to unintended endpoints.
- **Dependency vulnerabilities** — issues in upstream crates that affect Nova users.

Issues that require local code execution to exploit and offer no meaningful privilege escalation are generally considered out of scope, but feel free to report them if you are unsure.
