# Dockerfile that manually builds the entire app

# Frontend build stage
# =====================
FROM node:24-alpine AS frontend

WORKDIR /app

# Copy workspace metadata and frontend package manifest
COPY package.json ./
COPY pnpm-lock.yaml ./
COPY pnpm-workspace.yaml ./
COPY frontend/package.json ./frontend/package.json

# Install the repo dependencies using pnpm, like the GitHub workflow
RUN npm install -g pnpm@11.9.0 && pnpm install --frozen-lockfile

# Copy the remaining frontend source and build it
COPY frontend ./frontend
RUN pnpm --dir frontend run build

# Backend build stage
# =====================
FROM rust:1.93.1 AS backend

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

WORKDIR /app

# Copy cargo project files
COPY ./backend/Cargo.toml .
COPY ./backend/Cargo.lock .

# Create dummy contents for main source
RUN mkdir ./src && echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs

# Cargo build the dummy project for dependency caching
RUN cargo build --target x86_64-unknown-linux-musl --release

# Remove dummy src
RUN rm -rf ./src

# Copy real source code over
COPY ./backend/src ./src

# Copy the frontend over
COPY --from=frontend /app/frontend/dist ./public

# Update the modified time on the project files so they recompile
RUN touch -a -m ./src/main.rs

# Cargo build real source code
RUN cargo build --target x86_64-unknown-linux-musl --release

# Run stage
# =====================
FROM alpine:latest

# Copy our build
COPY --from=backend /app/target/x86_64-unknown-linux-musl/release/quizler ./

ENV QUIZLER_PORT=80
ENV RUST_LOG="quizler=info"

EXPOSE 80

CMD ["/quizler"]
