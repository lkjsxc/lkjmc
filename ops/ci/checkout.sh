#!/bin/bash
set -euo pipefail
umask 077
[[ ${CI_REPOSITORY:?} == lkjsxc/lkjmc-rebuild && ${CI_COMMIT:?} =~ ^[0-9a-f]{40}$ ]]
[[ -n ${CI_JOB_TOKEN:?} && ! -e .git && ! -L .git ]]
# Only an empty job workspace is accepted. No old config, hooks or LFS filters.
[[ -z $(find . -mindepth 1 -maxdepth 1 -print -quit) ]]
unset GIT_TRACE GIT_TRACE_CURL GIT_CURL_VERBOSE GIT_SSL_NO_VERIFY GIT_CONFIG_COUNT
export GIT_TERMINAL_PROMPT=0 GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_NO_REPLACE_OBJECTS=1
askpass=$(mktemp)
cleanup() { rm -f -- "$askpass"; unset CI_JOB_TOKEN; }
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP
cat >"$askpass" <<'ASKPASS'
#!/bin/sh
case "$1" in
  *Username*) printf '%s\n' x-access-token ;;
  *Password*) printf '%s\n' "$CI_JOB_TOKEN" ;;
  *) exit 1 ;;
esac
ASKPASS
chmod 0700 "$askpass"
export GIT_ASKPASS="$askpass"
git -c init.templateDir= init -q .
git config --local core.hooksPath /dev/null
git remote add origin https://forgejo.lkjsxc.com/lkjsxc/lkjmc-rebuild.git
# Ordinary fetch into a newly empty local repository; no force push is used.
git -c http.sslVerify=true -c http.followRedirects=false -c credential.helper= \
  fetch --quiet origin 'refs/heads/*:refs/remotes/origin/*' 'refs/tags/*:refs/tags/*'
git -c core.hooksPath=/dev/null checkout --quiet --detach "$CI_COMMIT"
[[ $(git rev-parse HEAD) == "$CI_COMMIT" ]]
[[ $(git rev-parse --is-shallow-repository) == false ]]
[[ $(git config --get remote.origin.url) == https://forgejo.lkjsxc.com/lkjsxc/lkjmc-rebuild.git ]]
