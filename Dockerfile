# syntax=docker/dockerfile:1
#- -------------------------------------------------------------------------------------------------
#- Global
#-
ARG DEBIAN_FRONTEND=noninteractive \
	TZ=${TZ:-Asia/Tokyo}

#- -------------------------------------------------------------------------------------------------
#- Base
#-
FROM rust:1.89.0-bookworm AS base
ARG DEBIAN_FRONTEND \
	TZ

SHELL [ "/bin/bash", "-c" ]

# set Timezone
RUN set -euxo pipefail && \
	ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Create user
RUN set -euxo pipefail && \
	groupadd --gid 60001 user && \
	useradd -s /bin/bash --uid 60001 --gid 60001 -m user && \
	echo user:password | chpasswd && \
	passwd -d user


#- -------------------------------------------------------------------------------------------------
#- Development
#-
FROM base AS dev
ARG DEBIAN_FRONTEND

RUN set -euxo pipefail && \
	apt-get -y update && \
	apt-get -y upgrade && \
	apt-get -y install --no-install-recommends \
	bash \
	ca-certificates \
	curl \
	git \
	gnupg \
	jq \
	nano \
	software-properties-common \
	sudo \
	wget \
	&& \
	\
	# Cleanup \
	apt-get -y autoremove && \
	apt-get -y clean && \
	rm -rf /var/lib/apt/lists/*

# Add sudo user
RUN set -euxo pipefail && \
	echo -e "user\tALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/user

# Add Biome latest install
RUN set -euxo pipefail && \
	curl -fSL -o /usr/local/bin/biome "$(curl -sfSL https://api.github.com/repos/biomejs/biome/releases/latest | \
	jq -r '.assets[] | select(.name | endswith("linux-x64")) | .browser_download_url')" && \
	chmod +x /usr/local/bin/biome && \
	type -p biome

# Install nodejs
RUN set -euxo pipefail && \
	mkdir -p /etc/apt/keyrings && \
	curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg  && \
	echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_22.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list  && \
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

# rust tools
RUN set -euxo pipefail && \
	# Bash completions
	mkdir -p                          /home/user/.local/share/bash-completion/completions && \
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
	cargo clippy --version && \
	cargo fmt --version && \
	rustc --version

# Install Claude Code
RUN set -euxo pipefail && \
	mkdir -p /home/user/.local/npm && \
	echo "prefix=/home/user/.local/npm" > /home/user/.npmrc && \
	echo -e "# local npm install\nexport PATH=\$PATH:\$HOME/.local/npm/bin" | tee -a /home/user/.bashrc && \
	echo -e "\n# Claude Code\nalias cc=\"claude --dangerously-skip-permissions\"" | tee -a /home/user/.bashrc && \
	npm install -g @anthropic-ai/claude-code && \
	exec $SHELL -l && \
	claude --version && \
	type cc

# vim: set filetype=dockerfile:
