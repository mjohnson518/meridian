# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

If you discover a security vulnerability in Meridian, please report it responsibly:

### Reporting Process

1. **Email**: Send details to security@meridian.finance (or the repository owner)
2. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact assessment
   - Suggested remediation (if any)

### What to Expect

- **Acknowledgment**: Within 48 hours of your report
- **Initial Assessment**: Within 5 business days
- **Status Updates**: At least weekly until resolution
- **Resolution Timeline**: Depends on severity (see below)

### Severity Levels and Response Times

| Severity | Description | Target Resolution |
|----------|-------------|-------------------|
| Critical | Remote code execution, key exposure | 24-48 hours |
| High | Authentication bypass, data exposure | 7 days |
| Medium | Information disclosure, CSRF | 30 days |
| Low | Minor issues, hardening | 90 days |

### Safe Harbor

We consider security research conducted in accordance with this policy to be:

- Authorized under the Computer Fraud and Abuse Act
- Exempt from DMCA provisions
- Conducted in good faith

We will not pursue legal action against researchers who:

- Follow responsible disclosure practices
- Avoid privacy violations and data destruction
- Do not exploit vulnerabilities beyond proof of concept

## Security Best Practices

When using Meridian:

### Environment Configuration

- Never commit `.env` files with real credentials
- Use strong, unique passwords for all services
- Rotate API keys and secrets regularly
- Enable 2FA where available

### Deployment Security

- Use HTTPS in production (enforced via HSTS)
- Configure proper CORS origins
- Enable rate limiting
- Monitor logs for suspicious activity

### Smart Contract Interactions

- Verify contract addresses before interaction
- Review transaction details before signing
- Use hardware wallets for significant holdings
- Stay updated on security advisories

## Known Issues

We track known security issues in our internal tracker. Critical issues affecting production are disclosed after patches are available.

## Recognition

We appreciate the security research community and maintain a hall of fame for responsible disclosures (with researcher permission).

## Contact

For security concerns: security@meridian.finance
For general questions: Open a GitHub discussion
