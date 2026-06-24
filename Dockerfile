FROM gradle:8.10.2-jdk21

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

WORKDIR /workspace
COPY . /workspace
RUN chmod +x /workspace/scripts/*.py /workspace/scripts/*.sh /workspace/gradlew

CMD ["./scripts/verify.sh"]
