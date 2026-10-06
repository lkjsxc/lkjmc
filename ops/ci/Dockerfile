FROM postgres@sha256:5a5a84b19854a9ffaa54082c166ff4ec27473a361e496e5ea167f298f2da9722 AS tools
USER root
RUN apt-get update && apt-get install --no-install-recommends -y build-essential pkg-config libssl-dev git python3 ca-certificates xz-utils unzip libnss3 libnspr4 libatk1.0-0 libatk-bridge2.0-0 libcups2 libdrm2 libxkbcommon0 libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libgbm1 libasound2t64 libpango-1.0-0 libcairo2 fonts-noto-cjk \
    && dpkg-query -W -f='${binary:Package}\t${Version}\n' > /opt/ci-packages.tsv \
    && rm -rf /var/lib/apt/lists/*
COPY toolchains /inputs/toolchains
COPY toolchains.lock.json install-toolchains.py /inputs/
RUN python3 /inputs/install-toolchains.py && rm -rf /inputs
COPY gitleaks /usr/local/bin/gitleaks
RUN printf '%s  /usr/local/bin/gitleaks\n' 88f91962aa2f93ac6ab281d553b9e125f5197bbbce38f9f2437f7299c32e5509 | sha256sum -c -
ENV PATH=/opt/ci/rust/bin:/opt/ci/toolchains/node/bin:/usr/lib/postgresql/18/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
ENV JAVA_HOME=/opt/ci/toolchains/java LKJMC_GRADLE=/opt/ci/toolchains/gradle/bin/gradle LKJMC_CARGO=/opt/ci/rust/bin/cargo
COPY vendor /opt/ci/vendor
COPY dependencies /inputs/dependencies
RUN cd /inputs/dependencies/web && npm ci --ignore-scripts --cache /opt/ci/npm-cache \
    && rm -rf node_modules
RUN cd /inputs/dependencies/tests/game && npm ci --ignore-scripts --cache /opt/ci/npm-cache \
    && rm -rf node_modules
RUN GRADLE_USER_HOME=/opt/ci/gradle-cache /opt/ci/toolchains/gradle/bin/gradle --no-daemon --max-workers=2 -p /inputs/dependencies/plugins \
    :paper:verifyDependencyBytes :proxy:verifyDependencyBytes :floodgate-link:verifyDependencyBytes :test-fixture:verifyDependencyBytes \
    && rm -rf /inputs /opt/ci/gradle-cache/daemon /opt/ci/gradle-cache/notifications
COPY game-artifacts /opt/ci/game-artifacts
COPY ux-public/browser /opt/ci/browsers/chromium_headless_shell-1243
COPY ux-public/paper-runtime /opt/ci/paper-runtime
COPY upload-artifact /opt/ci/upload-artifact
COPY check.py checkout.sh gitleaks.toml empty.gitleaksignore secret-policy.json secret_policy.py /opt/ci/lkjmc/
COPY toolchains.lock.json ux-inputs.lock.json /opt/ci/lkjmc/
RUN useradd --uid 1000 --create-home --home-dir /workspace --shell /bin/bash ci \
    && install -d -m 0755 -o ci -g ci /workspace /workspace/lkjsxc /workspace/lkjsxc/lkjmc \
    && chmod 0755 /opt/ci/lkjmc/checkout.sh \
    && find /opt/ci -type d -exec chmod a+rx {} + \
    && find /opt/ci -type f -exec chmod a+r {} + \
    && chmod 0755 /usr/local/bin/gitleaks \
    && postgres --version && rustc --version && node --version && gitleaks version
ENTRYPOINT []
USER ci
WORKDIR /workspace
CMD ["sleep", "infinity"]
