# syntax=docker/dockerfile:1
#- -------------------------------------------------------------------------------------------------
#- Global
#-
ARG DEBIAN_FRONTEND=noninteractive \
	TZ=${TZ:-Asia/Tokyo}

## renovate: datasource=github-releases packageName=edprint/dprint versioning=semver
ARG DPRINT_VERSION=0.50.0
## renovate: datasource=github-releases packageName=evilmartians/lefthook versioning=semver
ARG LEFTHOOK_VERSION=v1.12.2
## renovate: datasource=github-releases packageName=rui314/mold versioning=semver
ARG MOLD_VERSION=v2.40.3

# Rust tools
## renovate: datasource=github-releases packageName=taiki-e/cargo-llvm-cov versioning=semver
ARG CARGO_LLVM_COV_VERSION=v0.6.17

# retry dns and some http codes that might be transient errors
ARG CURL_OPTS="-sfSL --retry 3 --retry-delay 2 --retry-connrefused"


#- -------------------------------------------------------------------------------------------------
#- Builder Base
#-
FROM rust:1.89-trixie AS builder-base
ARG CARGO_LLVM_COV_VERSION \
	CURL_OPTS \
	DEBIAN_FRONTEND \
	MOLD_VERSION \
	TZ

ENV LC_ALL=C.utf8
ENV LANG=C.utf8

SHELL [ "/bin/bash", "-c" ]

RUN echo "**** set Timezone ****" && \
	set -euxo pipefail && \
	ln -snf /usr/share/zoneinfo/${TZ} /etc/localtime && echo ${TZ} > /etc/timezone

RUN echo "**** Dependencies ****" && \
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
	wget \
	&& \
	\
	# Cleanup \
	apt-get -y autoremove && \
	apt-get -y clean && \
	rm -rf /var/lib/apt/lists/*

RUN echo "**** Create user ****" && \
	set -euxo pipefail && \
	groupadd --gid 60001 user && \
	useradd -s /bin/bash --uid 60001 --gid 60001 -m user && \
	echo user:password | chpasswd && \
	passwd -d user

RUN echo "**** Add sudo user ****" && \
	set -euxo pipefail && \
	echo -e "user\tALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/user

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

# User level settings
USER user
RUN echo "**** Rust component ****" && \
	set -euxo pipefail && \
	# Bash completions
	mkdir -p                         /home/user/.local/share/bash-completion/completions && \
	rustup completions bash cargo  > /home/user/.local/share/bash-completion/completions/cargo && \
	rustup completions bash rustup > /home/user/.local/share/bash-completion/completions/rustup && \
	\
	# rustup toolchain cleanup
	rustup component add \
	cargo \
	clippy \
	llvm-tools \
	rust-analyzer \
	rust-docs \
	rust-std \
	rustc \
	rustfmt \
	&& \
	rustup component list --installed && \
	\
	rustup target add x86_64-unknown-linux-musl && \
	\
	cargo clippy --version && \
	cargo fmt --version && \
	rustc --version

USER root
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

RUN echo "**** Install Lefthook ****" && \
	set -euxo pipefail && \
	apt-get update && \
	_download_url="$(curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' \
	https://api.github.com/repos/evilmartians/lefthook/releases/tags/${LEFTHOOK_VERSION} | \
	jq -r '.assets[] | select(.name | endswith("_amd64.deb")) | .browser_download_url')" && \
	_filename="$(basename "$_download_url")" && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' -o "./${_filename}" "${_download_url}" && \
	ls -lah && \
	apt-get -y install "./${_filename}" && \
	\
	# Cleanup \
	apt-get -y autoremove && \
	apt-get -y clean && \
	rm -rf /var/lib/apt/lists/* && \
	\
	lefthook version --full && \
	lefthook completion bash > /home/user/.local/share/bash-completion/completions/lefthook && \
	rm -rf "./${_filename}"

RUN echo "**** Install nodejs for Claude Code ****" && \
	set -euxo pipefail && \
	mkdir -p /etc/apt/keyrings && \
	curl ${CURL_OPTS} -H 'User-Agent: builder/1.0' https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | \
	gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg  && \
	echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_22.x nodistro main" | \
	tee /etc/apt/sources.list.d/nodesource.list  && \
	apt-get update  && \
	apt-get -y install --no-install-recommends \
	nodejs \
	&& \
	\
	# Cleanup \
	apt-get -y autoremove && \
	apt-get -y clean && \
	rm -rf /var/lib/apt/lists/* && \
	node -v

# User level settings
USER user
RUN echo "**** Install Claude Code ****" && \
	set -euxo pipefail && \
	mkdir -p /home/user/.local/npm && \
	echo "prefix=/home/user/.local/npm" > /home/user/.npmrc && \
	echo -e "# local npm install\nexport PATH=\$PATH:\$HOME/.local/npm/bin" | tee -a /home/user/.bashrc && \
	echo -e "\n# Claude Code\nalias cc=\"claude --dangerously-skip-permissions\"" | tee -a /home/user/.bashrc && \
	npm install -g @anthropic-ai/claude-code && \
	exec ${SHELL} -l && \
	claude --version && \
	type cc


#- -------------------------------------------------------------------------------------------------
#- Production
#-
FROM debian:bullseye-slim
ARG DEBIAN_FRONTEND \
	TZ

SHELL [ "/bin/bash", "-c" ]

RUN echo "**** set Timezone ****" && \
	set -euxo pipefail && \
	ln -snf /usr/share/zoneinfo/${TZ} /etc/localtime && echo ${TZ} > /etc/timezone

RUN echo "**** Dependencies ****" && \
	set -euxo pipefail && \
	apt-get update && \
	apt-get -y install --no-install-recommends \
	bash \
	ca-certificates \
	&& \
	\
	# Cleanup \
	apt-get -y autoremove && \
	apt-get -y clean && \
	rm -rf /var/lib/apt/lists/*

#COPY --from=development /usr/local/cargo/bin/myapp /usr/local/bin/myapp

SHELL [ "/bin/sh", "-c" ]
#CMD ["myapp"]

# vim: set filetype=dockerfile:
