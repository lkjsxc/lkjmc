FROM rust:1.97.0-bullseye@sha256:7373627c1378e62a7e4963df5b6ceb90422c5b87afd59e17943029d19ea31e9a AS rust-toolchain

FROM gradle:8.10.2-jdk21@sha256:963d59f7f22767da4efbcf46b661361b61af5fb88b0309da1071c4234c647eba AS toolchain

USER root
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential python3 ca-certificates curl git unzip postgresql-client-14 \
    && dpkg-query -W -f='${Package}=${Version}\n' \
        build-essential python3 ca-certificates curl git unzip postgresql-client-14 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=rust-toolchain /usr/local/cargo /usr/local/cargo
COPY --from=rust-toolchain /usr/local/rustup /usr/local/rustup
ENV PATH="/usr/local/cargo/bin:${PATH}"
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
RUN rustc --version | grep -Fx 'rustc 1.97.0 (2d8144b78 2026-07-07)' \
    && cargo --version | grep -Fx 'cargo 1.97.0 (c980f4866 2026-06-30)'

FROM toolchain AS rust-deps
WORKDIR /workspace
COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo fetch --locked

FROM rust-deps AS gradle-deps
COPY settings.gradle.kts build.gradle.kts gradlew ./
COPY gradle ./gradle
COPY platforms ./platforms
RUN ./gradlew --no-daemon help >/dev/null

FROM gradle-deps AS compact-input
WORKDIR /
RUN rm -rf /opt/gradle /workspace \
        /home/gradle/.gradle/wrapper/dists/lkjmc \
        /usr/share/doc /usr/share/info /usr/share/man \
        /var/cache/apt /var/lib/apt/lists \
    && rm -f /usr/bin/gradle

FROM scratch AS compact-toolchain
COPY --from=compact-input / /
USER root
ENV DEBIAN_FRONTEND=noninteractive
ENV JAVA_HOME=/opt/java/openjdk
ENV LANG=en_US.UTF-8 LANGUAGE=en_US:en LC_ALL=en_US.UTF-8
ENV PATH=/usr/local/cargo/bin:/opt/java/openjdk/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
WORKDIR /workspace

FROM compact-toolchain AS verify
COPY . /workspace
RUN test -x /workspace/scripts/verify-full.sh \
    && test -x /workspace/scripts/attach-source-git.sh \
    && test -x /workspace/scripts/build-release.sh \
    && test -x /workspace/scripts/compare-release-roots.py \
    && test -x /workspace/scripts/release_archive.py \
    && test -x /workspace/scripts/private-artifact-handoff.py \
    && test -x /workspace/gradlew
CMD ["./scripts/verify-full.sh"]
