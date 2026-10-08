FROM rust:1.89-bookworm AS builder

WORKDIR /app

# Cache dependencies separately from application source changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && printf 'fn main() {}\n' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY src ./src
RUN touch src/main.rs \
    && cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 app

WORKDIR /app

COPY --from=builder /app/target/release/rhs-backend /usr/local/bin/rhs-backend

USER app

EXPOSE 8080

CMD ["rhs-backend"]
