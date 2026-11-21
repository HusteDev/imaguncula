FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

ARG NODE_VERSION=24.18.0
ARG PNPM_VERSION=11.9.0
ARG RUST_VERSION=1.94.1
ARG JUST_VERSION=1.57.0

USER root

RUN set -eux; \
    export DEBIAN_FRONTEND=noninteractive; \
    apt-get update; \
    apt-get install --yes --no-install-recommends \
      build-essential \
      ca-certificates \
      curl \
      file \
      libayatana-appindicator3-dev \
      librsvg2-dev \
      libssl-dev \
      libwebkit2gtk-4.1-dev \
      patchelf \
      pkg-config \
      wget \
      xdg-utils \
      xz-utils; \
    rm -rf /var/lib/apt/lists/*

RUN set -eux; \
    architecture="$(dpkg --print-architecture)"; \
    case "$architecture" in \
      amd64) node_architecture=x64 ;; \
      arm64) node_architecture=arm64 ;; \
      *) printf 'unsupported architecture: %s\n' "$architecture" >&2; exit 1 ;; \
    esac; \
    archive="node-v${NODE_VERSION}-linux-${node_architecture}.tar.xz"; \
    temporary_directory="$(mktemp -d)"; \
    cd "$temporary_directory"; \
    curl --proto '=https' --tlsv1.2 --fail --silent --show-error --location \
      "https://nodejs.org/dist/v${NODE_VERSION}/SHASUMS256.txt" --output SHASUMS256.txt; \
    curl --proto '=https' --tlsv1.2 --fail --silent --show-error --location \
      "https://nodejs.org/dist/v${NODE_VERSION}/${archive}" --output "$archive"; \
    grep " ${archive}$" SHASUMS256.txt | sha256sum --check --strict; \
    tar --extract --xz --file "$archive" --directory /usr/local --strip-components=1; \
    cd /; \
    rm -rf "$temporary_directory"; \
    npm install --global "pnpm@${PNPM_VERSION}"; \
    node --version; \
    pnpm --version

USER vscode
ENV PATH="/home/vscode/.cargo/bin:${PATH}"

RUN set -eux; \
    rustup_installer="$(mktemp)"; \
    curl --proto '=https' --tlsv1.2 --fail --silent --show-error --location \
      https://sh.rustup.rs --output "$rustup_installer"; \
    sh "$rustup_installer" -y --profile minimal --default-toolchain none; \
    rm -f "$rustup_installer"; \
    rustup toolchain install "$RUST_VERSION" --profile minimal --component clippy --component rustfmt; \
    rustup default "$RUST_VERSION"; \
    cargo install just --locked --version "$JUST_VERSION"; \
    rustc --version; \
    just --version

WORKDIR /workspaces/imaguncula
