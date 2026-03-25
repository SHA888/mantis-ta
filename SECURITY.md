# Security Policy

## Supported Versions

| Version | Supported |
|---------|------------|
| 0.5.x   | ✅ |
| 0.4.x   | ✅ |
| < 0.4.0 | ❌ |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it privately before disclosing it publicly.

### Reporting Methods

1. **GitHub Private Vulnerability Reporting** (Preferred):
   - Visit https://github.com/SHA888/mantis-ta/security/advisories
   - Click "Report a vulnerability"
   - Follow the prompts to submit your report

2. **Email**:
   - Send details to: [x@x.com](https://x.com/ks_sha888)
   - Include "SECURITY" in the subject line

3. **Issue Template**:
   - Use the [🚨 Security Vulnerability](https://github.com/SHA888/mantis-ta/issues/new?assignees=SHA888&labels=security%2Ctype%3Abug&template=security_vulnerability.md) issue template

### What to Include

Please include the following information in your report:

- **Affected versions**: Which versions are affected
- **Impact**: What is the impact of the vulnerability
- **Proof of concept**: A minimal reproduction case (if possible)
- **Mitigation**: Any suggested workarounds
- **Contact information**: How we can reach you for follow-up questions

## Response Timeline

- **Initial response**: Within 48 hours
- **Assessment**: Within 7 days
- **Fix timeline**: Depends on severity (see below)
- **Public disclosure**: After fix is released

## Severity Classification

| Severity | Fix Timeline | Examples |
|----------|--------------|-----------|
| Critical | 48 hours | Remote code execution, data exfiltration |
| High | 7 days | Privilege escalation, authentication bypass |
| Medium | 30 days | Information disclosure, DoS |
| Low | 90 days | Minor information leaks, UI issues |

## Coordinated Disclosure

We follow responsible disclosure practices:

1. Acknowledge receipt within 48 hours
2. Investigate and assess the vulnerability
3. Develop and test a fix
4. Release security update
5. Publish security advisory (with credit if desired)

## Security Best Practices

### For Users
- Keep dependencies updated (use Dependabot alerts)
- Review security advisories
- Use the latest stable version

### For Developers
- Use secure coding practices
- Regular dependency audits
- Security testing in CI/CD
- Monitor for vulnerability disclosures

## Dependencies

We use Dependabot to monitor for known vulnerabilities in dependencies. Security updates are prioritized and released as patches.

## Security Features

- **No unsafe code** in default build
- **Input validation** for all public APIs
- **Memory safety** through Rust's ownership system
- **Regular audits** of dependencies
- **Automated scanning** for known vulnerabilities

## Contact

For security-related questions:
- **Email**: security@mantis-ta.dev
- **GitHub**: @SHA888
- **Issues**: Use the security issue template

## Acknowledgments

We thank security researchers for helping us keep mantis-ta secure. All valid security reports will be acknowledged in our security advisories (with your permission).
