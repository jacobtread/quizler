<img src="assets/logo.svg" width="100%" height="80px">

# Quizler

![License](https://img.shields.io/github/license/jacobtread/Quizler?style=for-the-badge)
![Build](https://img.shields.io/github/actions/workflow/status/jacobtread/Quizler/build.yml?style=for-the-badge)

> Real time quiz application

## 📜 Table of Contents

- [🌟 Introduction](#-introduction)
- [📥 Downloads](#-downloads)
- [✨ Features](#-features)
- [📸 Screenshots](#-screenshots)
- [🛫 Deploying](#-deploying)
- [⚙️ Build Instructions](#-build-instructions)
- [🧾 License](#-license)

## 🌟 Introduction

Welcome to Quizler, an enhanced iteration of the original [Quizler](https://github.com/jacobtread/Quizler-v1) project. It's designed to be a real-time quiz application similar to [Kahoot](https://kahoot.com/), with a focus on being open-source and playable offline over LAN or self-hosted environments.

## 📥 Downloads

Prebuilt binaries are available for download:

| Platform             | Link                                                                                                       |
| -------------------- | ---------------------------------------------------------------------------------------------------------- |
| Windows              | [Download](https://github.com/jacobtread/Quizler/releases/latest/download/quizler-x86_64-windows-msvc.exe) |
| Linux (x86_64 GNU)   | [Download](https://github.com/jacobtread/Quizler/releases/latest/download/quizler-x86_64-linux-gnu)        |
| Linux (x86_64 MUSL)  | [Download](https://github.com/jacobtread/Quizler/releases/latest/download/quizler-x86_64-linux-musl)       |
| Linux (aarch64 GNU)  | [Download](https://github.com/jacobtread/Quizler/releases/latest/download/quizler-aarch64-linux-gnu)       |
| Linux (aarch64 MUSL) | [Download](https://github.com/jacobtread/Quizler/releases/latest/download/quizler-aarch64-linux-musl)      |

## 🐋 Docker

This repository includes the following Dockerfiles:

- `./containers/prebuilt.alpine.Dockerfile` - Prebuilt Alpine based image
- `./containers/prebuilt.debian.Dockerfile` - Prebuilt Debian based image
- `./containers/Build.Dockerfile` - Alpine based Builder image

Both the prebuilt alpine and debian releases are available from the Docker Hub registry and the Github Container Registry:

**Docker Hub Registry:**

`jacobtread/quizler:latest`

**Github Container Registry:**

`ghcr.io/jacobtread/quizler:latest`

The following tags are available with each image release:

- `latest` / `latest-debian` - Debian prebuilt of the latest Quizler version
- `latest-alpine` - Alpine prebuilt of the latest Quizler version
- `vX.X.X` / `vX.X.X-debian` - Debian prebuilt of a specific Quizler version
- `vX.X.X-alpine` - Alpine prebuilt of a specific Quizler version

## ✨ Features

- **Small and Performant**: Binary sizes for Windows and Linux are optimized, ensuring efficiency without sacrificing performance.
- **Name Filtering**: Maintain a positive environment by customizing the level of filtering for player usernames on a per-quiz basis.
- **Image Fitting**: Adapt quiz images for various devices with options like Cover, Contain, Fit Width, and Fit Height.
- **Customization Options**:
  - Adjust time limits for answering questions.
  - Set min, max, and bonus scores for each question.
  - Support for different question types: Single, Multiple, True/False, and Typer.
- **Easy Shuffling**: Quickly rearrange the order of questions and answers for variety.
- **Readiness and Preloading**: Ensure fairness by waiting for all players to preload images before starting question timers.
- **Fully Portable**: No files are persisted, making Quizler completely portable as a single executable. Quizzes can be exported/imported as self-contained .quizler files.
- **Containerized**: Run Quizler as a standalone executable or within a Docker container using the `jacobtread/quizler` container image. Example `docker-compose.yml` provided.
- **Short and Shareable Codes**: Quiz codes consist of short 5-character tokens (e.g., AZ2ES), making sharing easy.
- **Cross-Platform**: Server runs on both Windows and Linux, accessible from any device with a web browser.

## 📸 Screenshots

### Home screen

Allows users to select between joining a quiz and creating their own.

![Home Screen](assets/home.png)

### Create screen

For creating a new quiz.

![Create Screen](assets/create.png)

### Join screen

For joining a quiz.

![Join Screen](assets/join.png)

## ⚙️ Build Instructions

Building Quizler requires NodeJS (>=22) for the frontend and Cargo/Rust (>=1.93.0) for the backend:

> **Note**
> These commands are from within the context of the repository directory. If you haven't already download the repository and cd to the directory before running the commands below

1. **Frontend**:

   ```shell
   # Move to the frontend directory
   cd frontend
   # Install dependencies
   pnpm install
   # Run the dist commands
   pnpm run dist
   # Move out of the frontend directory
   cd ..
   ```

2. **Backend**:
   ```shell
   # Move to the backend directory
   cd backend
   # Build the release binary
   cargo build --release
   ```

After building the backend, find the binary at `backend/target/release/quizler[.exe]`.

For Docker-based builds, refer to the provided `Build.Dockerfile` in the repository, there is also
a docker image for pre-built binaries at `Dockerfile`.
