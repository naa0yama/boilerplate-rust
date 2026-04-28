#!/usr/bin/env bash
# .devcontainer/traefik.sh <command>
# Commands: setup | up | down | exec | status
#
# Must be run on the WSL2 host, not inside a devcontainer.
set -euo pipefail

readonly TRAEFIK_BIN="${HOME}/.local/bin/traefik"
readonly TRAEFIK_CONFIG="${HOME}/.config/traefik/traefik.yml"
readonly TRAEFIK_SERVICE="${HOME}/.config/systemd/user/traefik.service"
readonly TRAEFIK_PORT_ROUTER=8080
readonly TRAEFIK_PORT_DASHBOARD=8081

# ---------------------------------------------------------------------------
# Shared helpers
# ---------------------------------------------------------------------------

_host_check() {
	if [ "${MISE_ENV:-}" = "devcontainer" ] || [ -f "/.dockerenv" ]; then
		echo "Error: must be run on the WSL2 host, not inside a devcontainer." >&2
		exit 1
	fi
}

_workspace() {
	git rev-parse --show-toplevel
}

_project() {
	basename "$(_workspace)"
}

_branch() {
	local workspace raw
	workspace="$(_workspace)"
	raw="$(git -C "${workspace}" branch --show-current 2>/dev/null || true)"
	if [ -z "${raw}" ]; then
		raw="detached-$(git -C "${workspace}" rev-parse --short HEAD)"
	fi
	printf '%s' "${raw}" \
		| tr '[:upper:]' '[:lower:]' \
		| sed 's/[^a-z0-9-]/-/g; s/--*/-/g; s/^-//; s/-$//' \
		| cut -c1-63
}

# Strip single-line comments (//) from JSONC before passing to jq
_jsonc() {
	sed -e '/^[[:space:]]*\/\//d' -e 's|[[:space:]]//[^"]*$||g' "${1}"
}

_ports() {
	local devcontainer_json
	devcontainer_json="$(_workspace)/.devcontainer/devcontainer.json"
	_jsonc "${devcontainer_json}" | jq -r '.portsAttributes | keys[]' 2>/dev/null || true
}

_ensure_network() {
	docker network inspect devcontainer-traefik >/dev/null 2>&1 \
		|| docker network create devcontainer-traefik
}

# Query GitHub releases API for the latest traefik tag (e.g. "v3.4.1")
_traefik_latest() {
	curl -sfSL --retry 3 \
		https://api.github.com/repos/traefik/traefik/releases/latest \
		| jq -r '.tag_name'
}

# Print the installed traefik version tag (e.g. "v3.4.0"), or empty string
_traefik_installed() {
	if [ ! -x "${TRAEFIK_BIN}" ]; then
		echo ""
		return
	fi
	# "traefik version" output: "Version:      3.4.1" (variable spacing, no "v" prefix)
	"${TRAEFIK_BIN}" version 2>/dev/null \
		| grep -oE '[0-9]+\.[0-9]+\.[0-9]+' \
		| head -1 \
		| sed 's/^/v/' \
		|| echo ""
}

_traefik_install_version() {
	local version arch filename url tmpfile checksums_url checksums_file expected actual

	version="${1}"

	case "$(uname -m)" in
		x86_64)  arch="amd64" ;;
		aarch64) arch="arm64" ;;
		*)
			echo "Error: unsupported architecture $(uname -m)" >&2
			exit 1
			;;
	esac

	filename="traefik_${version}_linux_${arch}.tar.gz"
	url="https://github.com/traefik/traefik/releases/download/${version}/${filename}"
	checksums_url="https://github.com/traefik/traefik/releases/download/${version}/traefik_${version}_checksums.txt"

	tmpfile="$(mktemp)"
	checksums_file="$(mktemp)"

	echo "Downloading traefik ${version} (${arch})..."
	if ! curl -fSL --retry 3 --retry-delay 2 --retry-connrefused -o "${tmpfile}" "${url}"; then
		rm -f "${tmpfile}" "${checksums_file}"
		exit 1
	fi

	echo "Verifying checksum..."
	if ! curl -fSL --retry 3 --retry-delay 2 --retry-connrefused -o "${checksums_file}" "${checksums_url}"; then
		rm -f "${tmpfile}" "${checksums_file}"
		echo "Error: failed to download checksums file" >&2
		exit 1
	fi

	expected="$(grep "${filename}" "${checksums_file}" | awk '{print $1}')"
	actual="$(sha256sum "${tmpfile}" | awk '{print $1}')"
	rm -f "${checksums_file}"

	if [ -z "${expected}" ]; then
		rm -f "${tmpfile}"
		echo "Error: ${filename} not found in checksums file" >&2
		exit 1
	fi
	if [ "${expected}" != "${actual}" ]; then
		rm -f "${tmpfile}"
		echo "Error: checksum mismatch (expected ${expected}, got ${actual})" >&2
		exit 1
	fi

	mkdir -p "${HOME}/.local/bin"
	tar -xzf "${tmpfile}" -C "${HOME}/.local/bin" traefik
	rm -f "${tmpfile}"
	chmod +x "${TRAEFIK_BIN}"
	echo "Installed traefik ${version} to ${TRAEFIK_BIN}"
}

# Check installed version vs latest; install or update as needed.
# Prints one of: "installed", "updated", "already-latest"
_traefik_ensure_latest() {
	local installed latest
	installed="$(_traefik_installed)"
	latest="$(_traefik_latest)"

	if [ -z "${latest}" ]; then
		echo "Warning: could not fetch latest traefik version from GitHub." >&2
		if [ -z "${installed}" ]; then
			echo "Error: traefik is not installed and version check failed." >&2
			exit 1
		fi
		echo "already-latest"
		return
	fi

	if [ -z "${installed}" ]; then
		_traefik_install_version "${latest}"
		echo "installed"
	elif [ "${installed}" != "${latest}" ]; then
		echo "Updating traefik ${installed} -> ${latest}"
		_traefik_install_version "${latest}"
		echo "updated"
	else
		echo "traefik ${installed}: already at latest" >&2
		echo "already-latest"
	fi
}

# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

cmd_setup() {
	_host_check

	_traefik_ensure_latest >/dev/null
	# Show final installed version
	echo "traefik: $(_traefik_installed)"

	_ensure_network
	mkdir -p "$(dirname "${TRAEFIK_CONFIG}")" "$(dirname "${TRAEFIK_SERVICE}")"

	# "traefik" entrypoint name overrides Traefik v3's built-in default (:8080).
	# Without this, api.insecure creates an implicit "traefik" entrypoint at :8080
	# which conflicts with the "web" entrypoint on the same port.
	cat > "${TRAEFIK_CONFIG}" << YAML
entryPoints:
  web:
    address: ":${TRAEFIK_PORT_ROUTER}"
  traefik:
    address: ":${TRAEFIK_PORT_DASHBOARD}"
providers:
  docker:
    endpoint: "unix:///var/run/docker.sock"
    exposedByDefault: false
    network: devcontainer-traefik
api:
  dashboard: true
  insecure: true
YAML
	echo "Wrote ${TRAEFIK_CONFIG}"

	cat > "${TRAEFIK_SERVICE}" << UNIT
[Unit]
Description=Traefik reverse proxy for devcontainers
After=network.target

[Service]
ExecStart=${TRAEFIK_BIN} --configfile=${TRAEFIK_CONFIG}
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
UNIT
	echo "Wrote ${TRAEFIK_SERVICE}"

	systemctl --user daemon-reload
	systemctl --user enable traefik.service

	if systemctl --user is-active --quiet traefik.service; then
		echo "Restarting traefik to apply config..."
		systemctl --user restart traefik.service
	else
		systemctl --user start traefik.service
	fi

	systemctl --user status traefik.service --no-pager
	echo ""
	echo "Traefik router:    http://localhost:${TRAEFIK_PORT_ROUTER}"
	echo "Traefik dashboard: http://localhost:${TRAEFIK_PORT_DASHBOARD}/dashboard/"
}

cmd_up() {
	local workspace project branch ports run_args tmpfile result container_id

	_host_check

	# Check traefik binary exists; install/update if needed
	local update_result
	update_result="$(_traefik_ensure_latest)"
	case "${update_result}" in
		installed) echo "traefik installed: $(_traefik_installed)" ;;
		updated)   echo "traefik updated: $(_traefik_installed)" ;;
	esac

	cmd_down
	_ensure_network

	workspace="$(_workspace)"
	project="$(_project)"
	branch="$(_branch)"
	ports="$(_ports)"

	if [ -z "${ports}" ]; then
		echo "Warning: no portsAttributes found in devcontainer.json, skipping traefik labels." >&2
	fi

	# bt holds a literal backtick; using a variable avoids command substitution
	# inside double-quoted strings when building the traefik Host() rule.
	local bt router fqdn port
	bt='`'

	run_args='["--label=traefik.enable=true"'
	run_args="${run_args},\"--label=devcontainer.project=${project}\""
	run_args="${run_args},\"--label=devcontainer.branch=${branch}\""

	while IFS= read -r port; do
		[ -z "${port}" ] && continue
		router="p${port}-${branch}--${project}"
		fqdn="p${port}.${branch}.${project}.localhost"
		run_args="${run_args},\"--label=traefik.http.routers.${router}.rule=Host(${bt}${fqdn}${bt})\""
		run_args="${run_args},\"--label=traefik.http.routers.${router}.entrypoints=web\""
		run_args="${run_args},\"--label=traefik.http.services.${router}.loadbalancer.server.port=${port}\""
	done <<< "${ports}"

	run_args="${run_args}]"

	# --override-config replaces (not merges) devcontainer.json, so we must
	# merge our runArgs into the full base config before passing it.
	tmpfile="$(mktemp --suffix=.json)"
	# shellcheck disable=SC2064
	trap "rm -f '${tmpfile}'" EXIT
	_jsonc "${workspace}/.devcontainer/devcontainer.json" \
		| jq --argjson args "${run_args}" '. + {runArgs: $args}' \
		> "${tmpfile}"

	result="$(devcontainer up --workspace-folder "${workspace}" --override-config "${tmpfile}")"
	container_id="$(printf '%s' "${result}" | jq -r '.containerId')"

	if [ -z "${container_id}" ] || [ "${container_id}" = "null" ]; then
		echo "ERROR: failed to get containerId from devcontainer up" >&2
		printf '%s\n' "${result}" >&2
		exit 1
	fi

	if ! docker inspect "${container_id}" \
		--format '{{range $k,$v := .NetworkSettings.Networks}}{{$k}} {{end}}' \
		| grep -q devcontainer-traefik; then
		docker network connect devcontainer-traefik "${container_id}"
	fi

	# Splash
	echo ""
	echo "=================================================="
	echo "  ${project} / ${branch}"
	echo "=================================================="
	echo ""
	echo "  Container: ${container_id:0:12}"
	echo ""
	if [ -n "${ports}" ]; then
		echo "  URLs:"
		while IFS= read -r port; do
			[ -z "${port}" ] && continue
			echo "    http://p${port}.${branch}.${project}.localhost:${TRAEFIK_PORT_ROUTER}"
		done <<< "${ports}"
		echo ""
	fi
	echo "  Traefik router:    http://localhost:${TRAEFIK_PORT_ROUTER}"
	echo "  Traefik dashboard: http://localhost:${TRAEFIK_PORT_DASHBOARD}/dashboard/"
	echo ""
	echo "  Reconnect: mise run dev:exec"
	echo "=================================================="
	echo ""

	_exec_and_watch
}

cmd_down() {
	local workspace project branch container_id

	_host_check

	workspace="$(_workspace)"
	project="$(_project)"
	branch="$(_branch)"

	container_id="$(docker ps -aq \
		--filter "label=devcontainer.project=${project}" \
		--filter "label=devcontainer.branch=${branch}")"

	if [ -z "${container_id}" ]; then
		echo "No devcontainer found for ${project} / ${branch}"
		return 0
	fi

	docker rm -f "${container_id}"
	echo "Stopped: ${project} / ${branch} (${container_id})"
}

_exec_and_watch() {
	local workspace grace cancelled

	workspace="$(_workspace)"

	devcontainer exec --workspace-folder "${workspace}" bash || true

	grace=10
	cancelled=0
	trap 'cancelled=1' INT
	echo ""
	echo "Shell exited. Stopping container in ${grace}s... (Ctrl+C to cancel)"
	for i in $(seq "${grace}" -1 1); do
		printf "\r  %ds remaining..." "${i}"
		sleep 1
		if [ "${cancelled}" -eq 1 ]; then
			echo ""
			echo "Stop cancelled. Reconnect: mise run dev:exec"
			trap - INT
			return 0
		fi
	done
	echo ""
	trap - INT

	cmd_down
}

cmd_exec() {
	local project branch container_id

	_host_check

	project="$(_project)"
	branch="$(_branch)"

	container_id="$(docker ps -q \
		--filter "label=devcontainer.project=${project}" \
		--filter "label=devcontainer.branch=${branch}")"

	if [ -z "${container_id}" ]; then
		echo "Container not running. Starting..."
		cmd_up
	fi

	_exec_and_watch
}

cmd_status() {
	_host_check
	docker ps \
		--filter "label=traefik.enable=true" \
		--filter "label=devcontainer.project" \
		--format 'table {{.Names}}\t{{.Status}}\t{{.Label "devcontainer.project"}}\t{{.Label "devcontainer.branch"}}'
}

# ---------------------------------------------------------------------------
# Dispatch
# ---------------------------------------------------------------------------

case "${1:-}" in
	setup)  cmd_setup  ;;
	up)     cmd_up     ;;
	down)   cmd_down   ;;
	exec)   cmd_exec   ;;
	status) cmd_status ;;
	*)
		echo "Usage: $0 {setup|up|down|exec|status}" >&2
		exit 1
		;;
esac
