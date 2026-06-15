#!/usr/bin/env bash

SCRIPTS_DIR="${SCRIPTS_DIR:-$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)}"
BASE_DIR="${BASE_DIR:-$(dirname "${SCRIPTS_DIR}")}"

source "${SCRIPTS_DIR}/utils.sh"

function requested_device_id() {
    local index=0

    while [[ ${index} -lt ${#EXTRA_ARGS[@]} ]]; do
        case "${EXTRA_ARGS[$index]}" in
            -d|--device-id)
                if [[ $((index + 1)) -lt ${#EXTRA_ARGS[@]} ]]; then
                    echo "${EXTRA_ARGS[$((index + 1))]}"
                    return 0
                fi
                ;;
        esac
        index=$((index + 1))
    done

    return 1
}

function gitlab_api() {
    local endpoint="$1"
    local output
    local stderr_file
    local stderr_output

    stderr_file="$(mktemp "${TMPDIR:-/tmp}/nl-wallet-glab-api.XXXXXX")"

    if output="$(glab api "${endpoint}" 2>"${stderr_file}")"; then
        rm -f "${stderr_file}"
        echo "${output}"
        return 0
    fi

    local exit_code=$?
    stderr_output="$(<"${stderr_file}")"
    rm -f "${stderr_file}"

    case "${stderr_output}" in
        *"HTTP 401"*|*"HTTP 403"*)
            error "GitLab API authentication failed for '${endpoint}'."
            cat 1>&2 <<EOF
Check your existing glab login with:
  glab auth status

Or provide a token via:
  export GITLAB_TOKEN=<token>
EOF
            ;;
        *"HTTP 404"*)
            error "GitLab resource not found for '${endpoint}'."
            cat 1>&2 <<EOF
The selected pipeline or job does not expose the expected resource or artifact path.
Verify that the target pipeline was created after the relevant CI artifact change.
EOF
            ;;
        *)
            error "GitLab API request failed for '${endpoint}'."
            ;;
    esac

    if [[ -n "${stderr_output}" ]]; then
        echo "${stderr_output}" 1>&2
    fi

    exit "${exit_code}"
}

function resolve_jobs_json() {
    local project_id="$1"
    local pipeline_id="$2"

    gitlab_api "projects/${project_id}/pipelines/${pipeline_id}/jobs?scope[]=success&per_page=200"
}

function jobs_contain_name() {
    local jobs_json="$1"
    local job_name="$2"

    echo "${jobs_json}" | jq -e --arg name "${job_name}" '
        type == "array" and any(.[]; .name == $name)
    ' >/dev/null
}

function jobs_contain_all_names() {
    local jobs_json="$1"
    shift
    local job_name

    if ! echo "${jobs_json}" | jq -e 'type == "array"' >/dev/null 2>&1; then
        return 1
    fi

    for job_name in "$@"; do
        if ! jobs_contain_name "${jobs_json}" "${job_name}"; then
            return 1
        fi
    done

    return 0
}

function resolve_pipeline_id_for_jobs() {
    local project_id="$1"
    local gitlab_ref="$2"
    local pipeline_source="$3"
    local pipeline_description="$4"
    shift 4
    local required_jobs=("$@")
    local pipelines_json
    local pipeline_ids=()
    local pipeline_id
    local jobs_json

    pipelines_json="$(gitlab_api "projects/${project_id}/pipelines?ref=${gitlab_ref}&status=success&source=${pipeline_source}&per_page=100")"

    if ! echo "${pipelines_json}" | jq -e 'type == "array"' >/dev/null 2>&1; then
        error "Unexpected GitLab API response while resolving pipelines."
        echo "${pipelines_json}" 1>&2
        exit 1
    fi

    mapfile -t pipeline_ids < <(echo "${pipelines_json}" | jq -r '.[].id')
    if [[ ${#pipeline_ids[@]} -eq 0 ]]; then
        error "No successful ${pipeline_description} found for '${gitlab_ref}'."
        exit 1
    fi

    for pipeline_id in "${pipeline_ids[@]}"; do
        jobs_json="$(resolve_jobs_json "${project_id}" "${pipeline_id}")"
        if jobs_contain_all_names "${jobs_json}" "${required_jobs[@]}"; then
            echo "${pipeline_id}"
            return 0
        fi
    done

    error "No successful ${pipeline_description} found with ${required_jobs[*]}."
    cat 1>&2 <<EOF
Set PIPELINE_ID=<id> if you need to point the script at a specific pipeline.
EOF
    exit 1
}

function resolve_job_id() {
    local jobs_json="$1"
    local job_name="$2"

    echo "${jobs_json}" | jq -re --arg name "${job_name}" '
        if type != "array" then
            error("unexpected GitLab API response while resolving jobs")
        else
            map(select(.name == $name) | .id) | first // error("job not found: " + $name)
        end
    '
}

function fetch_job_artifact() {
    local project_id="$1"
    local job_id="$2"
    local artifact_path="$3"

    gitlab_api "projects/${project_id}/jobs/${job_id}/artifacts/${artifact_path}"
}

function fetch_wallet_config() {
    local project_id="$1"
    local job_id="$2"
    local wallet_config_dir="$3"
    local wallet_config_path="${wallet_config_dir}/wallet-config.json"
    local config_server_config_path="${wallet_config_dir}/config-server-config.json"

    mkdir -p "${wallet_config_dir}"

    # The wallet build script may create these ignored files as symlinks to tracked defaults.
    # Unlink symlink placeholders so fetched configs are written as real ignored files.
    [[ -L "${wallet_config_path}" ]] && rm -- "${wallet_config_path}"
    [[ -L "${config_server_config_path}" ]] && rm -- "${config_server_config_path}"

    fetch_job_artifact "${project_id}" "${job_id}" "wallet_core/wallet/wallet-config.json" \
        | jq -e '.' > "${wallet_config_path}"
    fetch_job_artifact "${project_id}" "${job_id}" "wallet_core/wallet/config-server-config.json" \
        | jq -e '.' > "${config_server_config_path}"
}

function require_env_vars() {
    local missing=()
    local var_name

    for var_name in "$@"; do
        if [[ -z "${!var_name:-}" ]]; then
            missing+=("${var_name}")
        fi
    done

    if [[ ${#missing[@]} -eq 0 ]]; then
        return 0
    fi

    error "Missing required environment value(s): ${missing[*]}"
    exit 1
}

function require_wrapper_entrypoint() {
    local helper_script_path="$1"
    local entrypoint_script="$2"
    local supported_entrypoints="$3"

    if [[ "${entrypoint_script}" != "${helper_script_path}" ]]; then
        return 0
    fi

    error "$(basename "${helper_script_path}") is an internal helper. Use ${supported_entrypoints}."
    exit 1
}
