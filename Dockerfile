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
ARG DPRINT_VERSION=0.50.1
## renovate: datasource=github-releases packageName=evilmartians/lefthook versioning=semver automerge=true
ARG LEFTHOOK_VERSION=v1.12.3
## renovate: datasource=github-releases packageName=rui314/mold versioning=semver automerge=true
ARG MOLD_VERSION=v2.40.4

# Rust tools
## renovate: datasource=github-releases packageName=cargo-bins/cargo-binstall versioning=semver automerge=true
ARG CARGO_BINSTALL_VERSION=v1.15.1
## renovate: datasource=github-releases packageName=taiki-e/cargo-llvm-cov versioning=semver automerge=true
ARG CARGO_LLVM_COV_VERSION=v0.6.18
## renovate: datasource=github-releases packageName=mozilla/sccache versioning=semver automerge=true
ARG SCCACHE_VERSION=v0.10.0

# retry dns and some http codes that might be transient errors
ARG CURL_OPTS="-sfSL --retry 3 --retry-delay 2 --retry-connrefused"


#- -------------------------------------------------------------------------------------------------
#- Builder Base
#-
FROM rust:1.89.0-trixie AS builder-base
ARG CARGO_BINSTALL_VERSION \
	CARGO_LLVM_COV_VERSION \
	CURL_OPTS \
	DEBIAN_FRONTEND \
	MOLD_VERSION \
	SCCACHE_VERSION \
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
	_download_url="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/rui314/mold/releases/tags/${MOLD_VERSION} | \
	jq -r '.assets[] | select(.name | endswith("-x86_64-linux.tar.gz")) | .browser_download_url')" && \
	_filename="$(basename "$_download_url")" && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
	tar -xvf "./${_filename}" --strip-components 1 -C /usr && \
	type -p mold && \
	rm -rf "./${_filename}"

RUN echo "**** Rust tool cargo-llvm-cov ****" && \
	set -euxo pipefail && \
	_download_url="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/taiki-e/cargo-llvm-cov/releases/tags/${CARGO_LLVM_COV_VERSION} | \
	jq -r '.assets[] | select(.name | endswith("-x86_64-unknown-linux-gnu.tar.gz")) | .browser_download_url')" && \
	_filename="$(basename "$_download_url")" && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
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

RUN --mount=type=bind,source=rust-toolchain.toml,target=/rust-toolchain.toml \
	\
	echo "**** Rust component ****" && \
	set -euxo pipefail && \
	cargo -V

RUN echo "**** Rust tools cargo-binstall ****" && \
	set -euxo pipefail && \
	_download_url="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/cargo-bins/cargo-binstall/releases/tags/${CARGO_BINSTALL_VERSION} | \
	jq -r '.assets[] | select(.name | endswith("-x86_64-unknown-linux-gnu.tgz")) | .browser_download_url')" && \
	_filename="$(basename "$_download_url")" && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
	tar -xvf "./${_filename}" -C /usr/local/bin/ && \
	type -p cargo-binstall && \
	rm -rf "./${_filename}"

# User level settings
USER ${USER_NAME}
ENV CARGO_HOME=/home/${USER_NAME}/.cargo

RUN echo "**** Create ${CARGO_HOME} ****" && \
	set -euxo pipefail && \
	mkdir -p "${CARGO_HOME}"

RUN echo "**** Rust tools ****" && \
	set -euxo pipefail && \
	cargo binstall --no-confirm \
	cargo-cache \
	cargo-modules

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
