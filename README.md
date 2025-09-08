# Simple Budget

Simple Budget is a full-stack web application that helps you manage your personal finances through a form of "envelope" saving. The application provides real-time financial insights, goal tracking, and comprehensive budget management through Accounts, Envelopes, Goals, and analytics.

## Architecture

- **Backend**: Axum web framework using Hotwire for HTML-over-the-wire
- **Frontend**: Server-side rendered HTML with TailwindCSS and Stimulus controllers
- **Database**: PostgreSQL
- **Authentication**: OpenID Connect (Google Sign-In)
- **Observability**: Jaeger tracing and Prometheus metrics

## Features

- **Multi-Account Management**: Track multiple bank accounts including assets and debts
- **Envelope Budgeting**: Allocate funds to different spending categories
- **Smart Goal Tracking**: Set and track financial goals with flexible recurrence patterns (daily, weekly, monthly, quarterly, yearly)
- **Real-time Analytics**: Per-day spending allowances, goal progress, and balance calculations
- **Automatic Goal Processing**: Background jobs handle recurring goal resets and accumulation
- **Timezone Support**: User-configurable timezone preferences

## Prerequisites

- Rust (2024 edition)
- PostgreSQL database
- Docker and Docker Compose
- Node.js (for TailwindCSS compilation)

## Quick Start

### Development Setup

1. **Clone the repository**

   ```bash
   git clone https://github.com/corybuecker/simple-budget
   cd simple-budget
   ```

2. **Start development services**

   ```bash
   cd dev && docker-compose up -d
   ```

3. **Set up environment variables**

   ```bash
   export DATABASE_URL="postgresql://simple_budget@localhost:5432/simple_budget"
   export SECURE="false"
   export GOOGLE_CALLBACK_URL="http://localhost:8000/authentication/callback"
   export GOOGLE_CLIENT_ID="<must be provided>"
   export GOOGLE_CLIENT_SECRET="<must be provided>"
   export SECRET_KEY="<must be provided>"
   ```

4. **Compile CSS**

   ```bash
   npx tailwindcss -i input.css -o static/app.css
   ```

5. **Start the development server**
   ```bash
   cargo run
   ```

### Auto-reload Development

```bash
cargo install cargo-watch
cargo watch -x run
```

For CSS changes, run TailwindCSS in watch mode:

```bash
npx tailwindcss -i input.css -o static/app.css --watch
```

## Development Services

The Docker Compose setup provides:

- **PostgreSQL**: Database server (port 5432)
- **Prometheus**: Metrics collection (port 9090)
- **Jaeger**: Distributed tracing (port 16686)

### Kubernetes Deployment

Apply the Kubernetes manifests:

```bash
kubectl apply -f k8s/
```

The application includes:

- Namespace isolation
- Resource limits (512M memory, 150m CPU)
- Health check endpoints
- Gateway API routing configuration

## Project Structure

```
├── src/
│   ├── models/              # Data models (Account, Envelope, Goal, User)
│   ├── routes/              # HTTP route handlers
│   ├── middleware/          # Authentication and CSRF middleware
│   ├── errors.rs            # Error handling and HTTP responses
│   └── main.rs              # Application entry point
├── migrations/              # Database schema migrations
├── assets/                  # Static assets and compiled CSS
├── templates_legacy/        # Tera HTML templates
├── k8s/                     # Kubernetes deployment manifests
├── ios/                     # Companion iOS application
└── dev/docker-compose.yaml  # Local development environment
```

## Database Schema

Core entities include:

- **users**: User profiles with OAuth integration
- **accounts**: Financial accounts (assets and debts)
- **envelopes**: Budget categories with allocated amounts
- **goals**: Financial targets with recurrence patterns
- **sessions**: Authentication sessions with CSRF protection

## Testing

Run the test suite:

```bash
cargo test
```

## Configuration

Required environment variables:

- `DATABASE_URL`: PostgreSQL connection string
- `SECRET_KEY`: Session signing key (generate securely for production)
- `SECURE`: Set to "true" in production for HTTPS cookies
- `GOOGLE_CALLBACK_URL`: Definition TBA
- `GOOGLE_CLIENT_ID`: Definition TBA
- `GOOGLE_CLIENT_SECRET`: Definition TBA

Optional:

- `LOG_LEVEL`: Logging verbosity, e.g. "debug"
- `METRICS_ENDPOINT`: Prometheus metrics export URL
- `TRACING_ENDPOINT`: Jaeger traces export URL

## License

MIT License - see LICENSE file for details.

## Notes

- This README was written by AI.
