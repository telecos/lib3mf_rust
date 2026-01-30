# Security Policy

## Supported Versions

We currently support the following versions with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in lib3mf_rust, please report it responsibly:

### How to Report

1. **Do NOT open a public issue** for security vulnerabilities
2. Instead, use one of these secure methods:
   - **GitHub Security Advisory**: Use the [Security Advisory](https://github.com/telecos/lib3mf_rust/security/advisories/new) feature
   - **Email**: Contact the maintainers directly at the email address listed in the repository

### What to Include

Please include the following information in your report:

- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact of the vulnerability
- Any suggested fixes (optional)
- Your contact information for follow-up questions

### Response Timeline

- **Initial Response**: We aim to acknowledge receipt within 48 hours
- **Status Updates**: We will provide updates on the investigation within 7 days
- **Fix Timeline**: Critical vulnerabilities will be addressed as soon as possible, typically within 30 days

### Disclosure Policy

- We request that you do not publicly disclose the vulnerability until we have had a chance to address it
- Once a fix is released, we will publicly acknowledge your responsible disclosure (unless you prefer to remain anonymous)
- We will credit you in the CHANGELOG and release notes

## Security Considerations

### Code Safety

This library is designed with security in mind:

- **No unsafe code**: The entire codebase uses `#![forbid(unsafe_code)]`
- **Memory safety**: All memory management handled by Rust's ownership system
- **Type safety**: Leverages Rust's type system for correctness
- **Input validation**: All XML and ZIP data is validated

### Known Security Considerations

#### 3MF File Parsing

- **Untrusted Input**: 3MF files should always be treated as untrusted input
- **XML Injection**: The parser validates XML structure and rejects malformed data
- **ZIP Bombs**: Large compressed files are handled, but extremely large files may consume significant memory
- **Resource Exhaustion**: Very large models may consume substantial memory and CPU

#### Secure Content Extension

- **Test Keys Only**: The library includes test decryption keys from the 3MF Consortium test suite
- **Production Use**: For production applications, use external cryptographic libraries with your own keys
- **Never Use Test Keys**: The embedded test keys are for conformance testing only and must not be used in production

#### Recommended Practices

When using lib3mf_rust in your application:

1. **Validate Input**: Always validate 3MF files from untrusted sources
2. **Resource Limits**: Consider imposing limits on file size and complexity
3. **Sandboxing**: Consider running 3MF parsing in a sandboxed environment for untrusted files
4. **Error Handling**: Always handle parsing errors gracefully
5. **Update Regularly**: Keep the library updated to receive security fixes

## Security Features

### Input Validation

The library performs comprehensive validation:

- XML structure validation
- Numeric range checking (triangle indices, vertex references)
- Resource ID validation and circular reference detection
- Color format validation
- UUID format validation
- File path validation within ZIP containers

### Safe Dependencies

We maintain vigilant dependency management:

- Regular dependency updates
- Security advisory monitoring
- Minimal dependency footprint
- Well-maintained, trusted dependencies only

## Acknowledgments

We appreciate the security research community's efforts in responsibly disclosing vulnerabilities. Contributors who report valid security issues will be acknowledged in our release notes (unless they prefer anonymity).

## Additional Resources

- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Secure Coding Practices](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/)
- [3MF Specification](https://3mf.io/specification/)
