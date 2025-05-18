# simple-web-healthcheck

![Crates.io Version](https://img.shields.io/crates/v/simple-web-healthcheck?style=for-the-badge)
![Crates.io Total Downloads](https://img.shields.io/crates/d/simple-web-healthcheck?style=for-the-badge)
![GitHub Release](https://img.shields.io/github/v/release/data5tream/simple-web-healthcheck-rs?label=GitHub&style=for-the-badge)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/data5tream/simple-web-healthcheck-rs/lint.yml?label=clippy&style=for-the-badge)
![GitHub License](https://img.shields.io/github/license/data5tream/simple-web-healthcheck-rs?style=for-the-badge&color=blue)

> Simple healthcheck for web app containers

## Usage

Provide the healthcheck URL as the first argument to the binary.

Check out the [web-test-container](https://github.com/Data5tream/web-test-container) Dockerfile for a usage example.

### Dockerfile

Add as a step to your multistage container build:

```dockerfile
# Build healthcheck
FROM rust:1.87-bookworm AS healthcheck-builder

RUN cargo install simple-web-healthcheck

# Rest of your build...
    
FROM gcr.io/distroless/cc-debian12

COPY --from=healthcheck-builder /usr/local/cargo/bin/simple-web-healthcheck /healthcheck

# Copy your app here

HEALTHCHECK --interval=10s --timeout=1s CMD ["/healthcheck", "http://127.0.0.1:8080"]
```
