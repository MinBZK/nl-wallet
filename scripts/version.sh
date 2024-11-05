#!/bin/bash

# Get the directory of the script.
script_dir="$(cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)"

# Navigate up one directory (assuming script is in project_dir/scripts).
project_root="$(cd "$script_dir/.." &> /dev/null && pwd)"

# Echo to stderr.
function error() {
  local msg=$1
  echo "$msg" 1>&2
}

# Check if required executables are available on the path.
function have() {
    local missing=()
    for executable in "$@"; do
        which "$executable" &>/dev/null || missing+=("$executable")
    done
    if [ ${#missing[@]} -eq 0 ]; then
        return 0
    else
        error "Missing required executable(s) to run this: ${missing[*]}"
        exit 1
    fi
}

# Check if cargo-edit is installed
# It is not sufficient to only have carge-set-version installed, and cargo-edit
# is not installed as a binary, so as heuristic we check all the subcommands.
function have_cargo_edit() {
    local missing=()
    for executable in cargo-add cargo-rm cargo-upgrade cargo-set-version; do
        which "$executable" &>/dev/null || missing+=("$executable")
    done
    if [ ${#missing[@]} -eq 0 ]; then
        return 0
    else
        error "Missing to run this: cargo-edit (missing: ${missing[*]})"
        error ""
        error "Install with: cargo install cargo-edit"
        error "          or: cargo binstall cargo-edit"
        exit 1
    fi
}

# Help output.
function help() {
    local cmd=$0
    error "Usage: $cmd [-h|-v|-c|-l|-g|-s <vX.Y.Z[-dev]>]"
    error ""
    error "This utility assists with managing the version of our components."
    error "It can get and set current version of components and assist with"
    error "handling git semver tags."
    error ""
    error "  -h            print this help message"
    error "  -v            show version of version.sh"
    error "  -c            show current version according to git"
    error "  -l            list released versions (git semver tags)"
    error "  -g            get current versions of components"
    error "  -s <vX.Y.Z>   set current versions of components"
    error ""
    exit 1;
}

# Show version.
function version() {
    echo 'v0.0.0'
    exit 0
}

# Current version according to git.
function current() {
    have git
    git describe --match 'v*.*.*' --tags
    exit $?
}

# List current semver tags in git.
function list() {
    have git
    git tag | grep '^v[0-9]*\.[0-9]*\.[0-9]*' | sort -V
    exit $?
}

# Get versions of our components as recorded in their respective project files.
function get() {
    have cat cargo flutter jq
    have_cargo_edit

    # Wallet core:
    echo "wallet_core: $(cd "$project_root/wallet_core" && cargo metadata --format-version=1 | jq -r '.packages[] | select(.name == "wallet") | .version')"

    # Wallet app:
    echo "wallet_app: $(cd "$project_root/wallet_app" && flutter pub deps --json | jq -r '.packages[] | select(.name == "wallet") | .version')"

    # Wallet web:
    echo "wallet_web: $(cd "$project_root/wallet_web" && cat package.json | jq -r '.version')"

    exit 0
}

# Set versions of our components in their respective project files.
function set() {
    have cargo flutter jq mv sed
    have_cargo_edit

    local version="$1"
    if [[ "$version" =~ ^(v)?[0-9]+\.[0-9]+\.[0-9]+(-dev)?$ ]]; then
        local non_prefixed_version="${version#v}"

        # Tell us about it.
        error "Setting versions of wallet_core, wallet_app and wallet_web to: $non_prefixed_version"

        # Wallet core (Cargo.toml):
        cargo set-version --manifest-path "$project_root/wallet_core/Cargo.toml" --workspace "$non_prefixed_version" > /dev/null 2>&1

        # Wallet app (pubspec.yaml):
        sed -i "s|^version:[\s]*.*$|version: $non_prefixed_version|g" "$project_root/wallet_app/pubspec.yaml" > /dev/null 2>&1

        # Wallet web (package.json):
        jq ".version = \"$non_prefixed_version\"" "$project_root/wallet_web/package.json" > "/tmp/wallet_web_package_json_$$" 2>&1
        mv "/tmp/wallet_web_package_json_$$" "$project_root/wallet_web/package.json" > /dev/null 2>&1

        # Inform about doing lock upgrades.
        error ""
        error "Note that at least npm's package-lock.json does not get updated with the"
        error "version that is now active in package.json, so you might also consider"
        error "upgrading your lock file. For completeness sake, here's how to do that"
        error "for our types of project (rust, flutter and node/javascript):"
        error ""
        error "    cd \"$project_root/wallet_core\" && cargo update"
        error "    cd \"$project_root/wallet_app\" && flutter pub upgrade"
        error "    cd \"$project_root/wallet_web\" && npm i --package-lock-only"
        error ""
        error "Caveat emptor! Of course assuming you have sane version specs in your"
        error "Cargo.toml, pubspec.yaml and/or package.json, and assuming you know what"
        error "you're doing when your're upgrading possibly *all* your dependencies!"

        exit 0
    else
        error "Invalid version number specified: $version"
        exit 1
    fi
}

# Getopt case.
while getopts 'hvclgs:' o; do
    case "$o" in
        h)
            help
            ;;
        v)
            version
            ;;
        c)
            current
            ;;
        l)
            list
            ;;
        g)
            get
            ;;
        s)
            set "$OPTARG"
            ;;
        *)
            help
            ;;
    esac
done

# If any case above fell through to here, print help.
shift $((OPTIND-1))
help
