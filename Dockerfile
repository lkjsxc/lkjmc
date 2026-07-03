FROM gradle:8.10.2-jdk21 AS toolchain

USER root
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        python3 \
        ca-certificates \
        curl \
        unzip \
    && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --component rustfmt --component clippy
ENV PATH="/root/.cargo/bin:${PATH}"

FROM toolchain AS rust-deps
WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo fetch

FROM rust-deps AS gradle-deps
COPY settings.gradle.kts build.gradle.kts gradlew ./
COPY gradle ./gradle
COPY platforms ./platforms
RUN ./gradlew --no-daemon help >/dev/null

FROM gradle-deps AS verify
COPY . /workspace
RUN chmod +x /workspace/scripts/*.py /workspace/scripts/*.sh /workspace/gradlew
CMD ["./scripts/verify-full.sh"]

FROM verify AS playable
CMD ["./scripts/compose-playable-entrypoint.sh"]
