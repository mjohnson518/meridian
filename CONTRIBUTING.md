# Contributing to Meridian

Thank you for your interest in contributing to Meridian. This document outlines the contribution process.

## Development Setup

### Prerequisites

- Rust 1.70+
- Node.js 18+
- PostgreSQL 14+
- Foundry (for smart contract development)

### Local Development

```bash
# Clone the repository
git clone https://github.com/mjohnson518/meridian.git
cd meridian

# Install Rust dependencies
cargo build

# Install frontend dependencies
cd meridian-frontend
npm install

# Copy environment template
cp .env.example .env
# Edit .env with your local configuration

# Run database migrations
sqlx migrate run

# Start the API server
cargo run --package meridian-api

# Start the frontend (in another terminal)
cd meridian-frontend
npm run dev
```

## Code Standards

### Rust

- Follow Rust API guidelines
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Write tests for new functionality

### TypeScript/React

- Use TypeScript strict mode
- Follow React best practices
- Ensure `npm run lint` passes
- Ensure `npm run type-check` passes

### Smart Contracts

- Follow Solidity style guide
- Write comprehensive test coverage
- Document all public functions
- Run `forge fmt` before committing

## Pull Request Process

1. Fork the repository and create a feature branch
2. Make your changes with clear, descriptive commits
3. Ensure all tests pass (`cargo test`, `npm test`, `forge test`)
4. Update documentation if applicable
5. Submit a pull request with a clear description

### PR Requirements

- All CI checks must pass
- Code review approval required
- No merge conflicts with main branch
- Commits should be atomic and well-described

## Commit Messages

Use conventional commit format:

```
type(scope): description

[optional body]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat(api): add rate limiting middleware`
- `fix(frontend): resolve auth token refresh loop`
- `docs(readme): update deployment instructions`

## Reporting Issues

- Check existing issues before creating new ones
- Use issue templates when available
- Include reproduction steps for bugs
- Provide environment details (OS, versions)

## Security Vulnerabilities

**Do not report security vulnerabilities through public issues.**

See [SECURITY.md](SECURITY.md) for responsible disclosure instructions.

## Questions

For questions about contributing, open a discussion or reach out to the maintainers.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
