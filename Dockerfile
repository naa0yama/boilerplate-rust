# syntax=docker/dockerfile:1
#- -------------------------------------------------------------------------------------------------
#- Global
#-
ARG DEBIAN_FRONTEND=noninteractive \
	TZ=${TZ:-Asia/Tokyo} \
	USER_NAME=cuser \
	USER_UID=${USER_UID:-60001} \
	USER_GID=${USER_GID:-${USER_UID}}

## renovate: datasource=github-releases packageName=dprint/dprint versioning=semver automerge=true
ARG DPRINT_VERSION=0.50.2
## renovate: datasource=github-releases packageName=evilmartians/lefthook versioning=semver automerge=true
ARG LEFTHOOK_VERSION=v1.13.0
## renovate: datasource=github-releases packageName=rui314/mold versioning=semver automerge=true
ARG MOLD_VERSION=v2.40.4

# Rust tools
## renovate: datasource=github-releases packageName=ast-grep/ast-grep versioning=semver automerge=true
ARG AST_GREP_VERSION=0.39.5
## renovate: datasource=github-tags packageName=matthiaskrgr/cargo-cache versioning=semver automerge=true
ARG CACHE_VERSION=0.8.3
## renovate: datasource=github-tags packageName=regexident/cargo-modules versioning=semver automerge=true
ARG MODULES_VERSION=v0.24.3
## renovate: datasource=github-releases packageName=casey/just versioning=semver automerge=true
ARG JUST_VERSION=1.42.4
## renovate: datasource=github-releases packageName=taiki-e/cargo-llvm-cov versioning=semver automerge=true
ARG LLVM_COV_VERSION=v0.6.19
## renovate: datasource=github-releases packageName=mozilla/sccache versioning=semver automerge=true
ARG SCCACHE_VERSION=v0.10.0
## renovate: datasource=github-releases packageName=ziglang/zig versioning=semver automerge=true
ARG ZIG_VERSION=0.15.1
## renovate: datasource=github-releases packageName=rust-cross/cargo-zigbuild versioning=semver automerge=true
ARG ZIGBUILD_VERSION=v0.20.1

# retry dns and some http codes that might be transient errors
ARG CURL_OPTS="-sfSL --retry 3 --retry-delay 2 --retry-connrefused"


#- -------------------------------------------------------------------------------------------------
#- Builder Base
#-
FROM rust:1.89.0-trixie AS builder-base
ARG AST_GREP_VERSION \
	CACHE_VERSION \
	MODULES_VERSION \
	CURL_OPTS \
	DEBIAN_FRONTEND \
	JUST_VERSION \
	LLVM_COV_VERSION \
	MOLD_VERSION \
	SCCACHE_VERSION \
	ZIG_VERSION \
	ZIGBUILD_VERSION \
	USER_NAME \
	USER_UID \
	USER_GID \
	TZ

ENV LANG=C.utf8 LC_ALL=C.utf8

SHELL [ "/bin/bash", "-c" ]

RUN echo "**** set Timezone ****" && \
	set -euxo pipefail && \
	ln -snf /usr/share/zoneinfo/${TZ} /etc/localtime && echo ${TZ} > /etc/timezone

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
	--mount=type=cache,target=/var/lib/apt,sharing=locked \
	\
	echo "**** Dependencies ****" && \
	rm -f /etc/apt/apt.conf.d/docker-clean && \
	echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache && \
	echo "**** Dependencies ****" && \
	set -euxo pipefail && \
	apt-get -y update && \
	apt-get -y upgrade && \
	apt-get -y install --no-install-recommends \
	bash \
	bash-completion \
	ca-certificates \
	curl \
	git \
	gnupg \
	jq \
	musl-tools \
	nano \
	sudo \
	wget

RUN echo "**** Create user ****" && \
	set -euxo pipefail && \
	groupadd --gid "${USER_GID}" "${USER_NAME}" && \
	useradd -s /bin/bash --uid "${USER_UID}" --gid "${USER_GID}" -m "${USER_NAME}" && \
	echo "${USER_NAME}:password" | chpasswd && \
	passwd -d "${USER_NAME}"

RUN echo "**** Add sudo user ****" && \
	set -euxo pipefail && \
	echo -e "${USER_NAME}\tALL=(ALL) NOPASSWD:ALL" > "/etc/sudoers.d/${USER_NAME}"

RUN echo "**** Install mold ****" && \
	set -euxo pipefail && \
	_release_data="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/rui314/mold/releases/tags/${MOLD_VERSION})" && \
	_asset="$(echo "$_release_data" | jq -r '.assets[] | select(.name | endswith("-x86_64-linux.tar.gz"))')" && \
	_download_url="$(echo "$_asset" | jq -r '.browser_download_url')" && \
	_digest="$(echo "$_asset" | jq -r '.digest')" && \
	_sha256="${_digest#sha256:}" && \
	_filename="$(basename "$_download_url")" && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
	echo "${_sha256}  ${_filename}" | sha256sum -c - && \
	tar -xvf "./${_filename}" --strip-components 1 -C /usr && \
	type -p mold && \
	rm -rf "./${_filename}"

RUN echo "**** Rust tool ast-grep ****" && \
	set -euxo pipefail && \
	_release_data="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/ast-grep/ast-grep/releases/tags/${AST_GREP_VERSION})" && \
	_asset="$(echo "$_release_data" | jq -r '.assets[] | select(.name | endswith("x86_64-unknown-linux-gnu.zip"))')" && \
	_download_url="$(echo "$_asset" | jq -r '.browser_download_url')" && \
	_digest="$(echo "$_asset" | jq -r '.digest')" && \
	_sha256="${_digest#sha256:}" && \
	_filename="$(basename "$_download_url")" && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
	echo "${_sha256}  ${_filename}" | sha256sum -c - && \
	unzip "./${_filename}" -d "/usr/local/bin/" && \
	type -p ast-grep sg && \
	rm -rf "./${_filename}"

RUN echo "**** Rust tool just ****" && \
	set -euxo pipefail && \
	_release_data="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/casey/just/releases/tags/${JUST_VERSION})" && \
	_asset="$(echo "$_release_data" | jq -r '.assets[] | select(.name | endswith("-x86_64-unknown-linux-musl.tar.gz"))')" && \
	_download_url="$(echo "$_asset" | jq -r '.browser_download_url')" && \
	_digest="$(echo "$_asset" | jq -r '.digest')" && \
	_sha256="${_digest#sha256:}" && \
	_filename="$(basename "$_download_url")" && \
	_tmpdir=$(mktemp -q -d) && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
	echo "${_sha256}  ${_filename}" | sha256sum -c - && \
	tar -xvf "./${_filename}" -C "${_tmpdir}" && \
	ls -lah "${_tmpdir}" && \
	cp -av "${_tmpdir}/just" /usr/local/bin/ && \
	cp -av "${_tmpdir}/completions/just.bash" /usr/share/bash-completion/completions/ && \
	type -p just && \
	rm -rf "./${_filename}" "${_tmpdir}"

RUN echo "**** Rust tool cargo-llvm-cov ****" && \
	set -euxo pipefail && \
	_release_data="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/taiki-e/cargo-llvm-cov/releases/tags/${LLVM_COV_VERSION})" && \
	_asset="$(echo "$_release_data" | jq -r '.assets[] | select(.name | endswith("-x86_64-unknown-linux-gnu.tar.gz"))')" && \
	_download_url="$(echo "$_asset" | jq -r '.browser_download_url')" && \
	_digest="$(echo "$_asset" | jq -r '.digest')" && \
	_sha256="${_digest#sha256:}" && \
	_filename="$(basename "$_download_url")" && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
	echo "${_sha256}  ${_filename}" | sha256sum -c - && \
	tar -xvf "./${_filename}" -C /usr/local/bin/ && \
	type -p cargo-llvm-cov && \
	rm -rf "./${_filename}"

RUN echo "**** Rust tool sccache ****" && \
	set -euxo pipefail && \
	_download_url="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/mozilla/sccache/releases/tags/${SCCACHE_VERSION} | \
	jq -r '.assets[] | select(.name | startswith("sccache-v") and endswith("-x86_64-unknown-linux-musl.tar.gz")) | .browser_download_url')" && \
	_filename="$(basename "$_download_url")" && \
	_tmpdir=$(mktemp -q -d) && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
	tar -xvf "./${_filename}" --strip-components 1 -C "${_tmpdir}" && \
	ls -lah "${_tmpdir}" && \
	cp -av "${_tmpdir}/sccache" /usr/local/bin/ && \
	type -p sccache && \
	rm -rf "./${_filename}" "${_tmpdir}"

RUN echo "**** Rust tool zig ****" && \
	set -euxo pipefail && \
	_filename="zig-x86_64-linux-${ZIG_VERSION}.tar.xz" && \
	_tmpdir=$(mktemp -q -d) && \
	mkdir -p /usr/local/zig && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" \
	"https://ziglang.org/download/${ZIG_VERSION}/zig-x86_64-linux-${ZIG_VERSION}.tar.xz" && \
	tar -xf "./${_filename}" --strip-components 1 -C "/usr/local/zig/" && \
	ls -lah /usr/local/zig && \
	rm -rf "./${_filename}" "${_tmpdir}"

RUN echo "**** Rust tool cargo-zigbuild ****" && \
	set -euxo pipefail && \
	_release_data="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/rust-cross/cargo-zigbuild/releases/tags/${ZIGBUILD_VERSION})" && \
	_asset="$(echo "$_release_data" | jq -r '.assets[] | select(.name | startswith("cargo-zigbuild-v") and endswith("x86_64-unknown-linux-musl.tar.gz"))')" && \
	_download_url="$(echo "$_asset" | jq -r '.browser_download_url')" && \
	_digest="$(echo "$_asset" | jq -r '.digest')" && \
	_sha256="${_digest#sha256:}" && \
	_filename="$(basename "$_download_url")" && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
	echo "${_sha256}  ${_filename}" | sha256sum -c - && \
	tar -xvf "./${_filename}" -C /usr/local/bin/ && \
	type -p cargo-zigbuild && \
	rm -rf "./${_filename}"

RUN --mount=type=bind,source=rust-toolchain.toml,target=/rust-toolchain.toml \
	\
	echo "**** Rust component ****" && \
	set -euxo pipefail && \
	cargo -V

# User level settings
USER ${USER_NAME}
ENV CARGO_HOME=/home/${USER_NAME}/.cargo

RUN echo "**** PATH add zig ****" && \
	set -euxo pipefail && \
	echo -e "# Add PATH ziglang\nexport PATH="/usr/local/zig:\$PATH"" >> ~/.bashrc && \
	exec ${SHELL} -l && \
	zig version

RUN echo "**** Create ${CARGO_HOME} ****" && \
	set -euxo pipefail && \
	mkdir -p "${CARGO_HOME}"

RUN --mount=type=cache,target=/home/cuser/.cache/sccache,sharing=locked,uid=${USER_UID},gid=${USER_GID} \
	--mount=type=cache,target=/home/cuser/.cargo/registry,sharing=locked,uid=${USER_UID},gid=${USER_GID} \
	\
	echo "**** Rust tools ****" && \
	set -euxo pipefail && \
	cargo install \
	cargo-cache@${CACHE_VERSION} \
	cargo-llvm-cov@${LLVM_COV_VERSION#v} \
	cargo-modules@${MODULES_VERSION#v} \
	&& \
	cargo cache --version && \
	cargo modules --version

RUN echo "**** Rust bash-completion ****" && \
	set -euxo pipefail && \
	echo "export PATH="\$CARGO_HOME/bin:\$PATH"" >> ~/.bashrc && \
	\
	mkdir -p                         /home/${USER_NAME}/.local/share/bash-completion/completions && \
	rustup completions bash cargo  > /home/${USER_NAME}/.local/share/bash-completion/completions/cargo && \
	rustup completions bash rustup > /home/${USER_NAME}/.local/share/bash-completion/completions/rustup

USER root


#- -------------------------------------------------------------------------------------------------
#- Development
#-
FROM builder-base AS development
ARG CURL_OPTS \
	DEBIAN_FRONTEND \
	DPRINT_VERSION \
	LEFTHOOK_VERSION

RUN echo "**** Install dprint ****" && \
	set -euxo pipefail && \
	_download_url="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/dprint/dprint/releases/tags/${DPRINT_VERSION} | \
	jq -r '.assets[] | select(.name | endswith("x86_64-unknown-linux-gnu.zip")) | .browser_download_url')" && \
	_filename="$(basename "$_download_url")" && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
	unzip "${_filename}" -d /usr/local/bin/ && \
	type -p dprint && \
	rm -rf "./${_filename}"

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
	--mount=type=cache,target=/var/lib/apt,sharing=locked \
	\
	echo "**** Install Lefthook ****" && \
	set -euxo pipefail && \
	_download_url="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/evilmartians/lefthook/releases/tags/${LEFTHOOK_VERSION} | \
	jq -r '.assets[] | select(.name | endswith("_amd64.deb")) | .browser_download_url')" && \
	_filename="$(basename "$_download_url")" && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
	ls -lah && \
	apt-get -y install "./${_filename}" && \
	\
	lefthook version --full && \
	rm -rf "./${_filename}"

# User level settings
USER ${USER_NAME}
RUN echo "**** add Lefthook bash-completion ****" && \
	set -euxo pipefail && \
	lefthook completion bash > /home/${USER_NAME}/.local/share/bash-completion/completions/lefthook

# Ref: https://docs.anthropic.com/en/docs/claude-code/setup#native-binary-installation-beta
RUN echo "**** Install Claude Code ****" && \
	set -euxo pipefail && \
	curl -fsSL https://claude.ai/install.sh | bash && \
	echo -e "\n# Claude Code\nexport PATH=\"\$HOME/.local/bin:\$PATH\"\nalias cc=\"claude --dangerously-skip-permissions\"" | tee -a "/home/${USER_NAME}/.bashrc" && \
	exec ${SHELL} -l && \
	claude --version && \
	type cc


#- -------------------------------------------------------------------------------------------------
#- Production
#-
#FROM debian:bullseye-slim
#ARG DEBIAN_FRONTEND \
#	TZ
#
#SHELL [ "/bin/bash", "-c" ]
#
#RUN echo "**** set Timezone ****" && \
#	set -euxo pipefail && \
#	ln -snf /usr/share/zoneinfo/${TZ} /etc/localtime && echo ${TZ} > /etc/timezone
#
#RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
#	--mount=type=cache,target=/var/lib/apt,sharing=locked \
#	\
#	echo "**** Dependencies ****" && \
#	set -euxo pipefail && \
#	apt-get -y install --no-install-recommends \
#	bash \
#	ca-certificates
#
##COPY --from=development /usr/local/cargo/bin/myapp /usr/local/bin/myapp
#
#SHELL [ "/bin/sh", "-c" ]
##CMD ["myapp"]

# vim: set filetype=dockerfile:
