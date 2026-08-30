FROM ubuntu:24.04@sha256:1e0a86e57d247923571b75e0aaf48a1449cf8c543d51fb3e07a4a7d7bfa79316

ARG DEBIAN_FRONTEND=noninteractive
ENV container=docker
ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8

COPY runtime-packages.lock /tmp/runtime-packages.lock
RUN printf '%s\n' \
      'Types: deb' \
      'URIs: https://snapshot.ubuntu.com/ubuntu/20260830T000000Z/' \
      'Suites: noble noble-updates noble-security' \
      'Components: main universe' \
      'Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg' \
      'Check-Valid-Until: no' \
      > /etc/apt/sources.list.d/ubuntu.sources \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get -o Acquire::https::Verify-Peer=false update -qq \
    && certificate_package="$(awk -F= '$1 == "ca-certificates" { print $0 }' /tmp/runtime-packages.lock)" \
    && test -n "$certificate_package" \
    && apt-get -o Acquire::https::Verify-Peer=false install -y --no-install-recommends "$certificate_package" \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get update -qq \
    && mkdir -p /etc/postgresql-common \
    && printf 'create_main_cluster = false\n' > /etc/postgresql-common/createcluster.conf \
    && printf '#!/bin/sh\nexit 101\n' > /usr/sbin/policy-rc.d \
    && chmod 0755 /usr/sbin/policy-rc.d \
    && xargs -r apt-get install -y --no-install-recommends < /tmp/runtime-packages.lock \
    && dpkg-query -W -f='${Package}=${Version}\n' | LC_ALL=C sort > /usr/share/lkjmc-docker-release-recovery-packages.txt \
    && rm -f /usr/sbin/policy-rc.d \
    && rm -rf /var/lib/apt/lists/* /tmp/runtime-packages.lock \
    && test "$(readlink -f /sbin/init)" = /usr/lib/systemd/systemd \
    && test "$(java -version 2>&1 | sed -n '1s/.*version "\([0-9][0-9]*\).*/\1/p')" = 21 \
    && test "$(pg_dump --version | sed -n 's/.* \([0-9][0-9]*\)\..*/\1/p')" = 16 \
    && test -z "$(pg_lsclusters --no-header)" \
    && printf '' > /etc/machine-id \
    && ln -sf /etc/machine-id /var/lib/dbus/machine-id

COPY lkjmc-lab-probe.service /etc/systemd/system/lkjmc-lab-probe.service
COPY systemd-entrypoint /usr/local/libexec/lkjmc-drr-systemd-entrypoint
COPY fixture.py /usr/local/libexec/lkjmc-drr-fixture
RUN ln -s /etc/systemd/system/lkjmc-lab-probe.service \
      /etc/systemd/system/multi-user.target.wants/lkjmc-lab-probe.service \
    && chmod 0755 \
      /usr/local/libexec/lkjmc-drr-fixture \
      /usr/local/libexec/lkjmc-drr-systemd-entrypoint \
    && printf 'schemaVersion=1\npurpose=disposable-docker-release-recovery-fixture\n' \
      > /usr/share/lkjmc-drr-image \
    && chmod 0444 /usr/share/lkjmc-drr-image \
    && systemctl mask \
      console-getty.service \
      getty@.service \
      systemd-logind.service \
      systemd-remount-fs.service \
      systemd-udevd.service \
      systemd-udevd-control.socket \
      systemd-udevd-kernel.socket

LABEL io.lkjmc.docker-release-recovery.image="systemd-host-v1"
LABEL io.lkjmc.docker-release-recovery.base="ubuntu-24.04-amd64@sha256:1e0a86e57d247923571b75e0aaf48a1449cf8c543d51fb3e07a4a7d7bfa79316"
LABEL io.lkjmc.docker-release-recovery.snapshot="ubuntu-20260830T000000Z"

STOPSIGNAL SIGRTMIN+3
CMD ["/usr/local/libexec/lkjmc-drr-systemd-entrypoint"]
