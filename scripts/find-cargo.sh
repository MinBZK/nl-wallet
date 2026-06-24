# source this file

check_cargo() {
    if command -v cargo; then
        return
    fi
    if [[ -n $1 ]]; then
        fail "$1"
    fi
    return 1
}

fail() {
    >&2 echo "ERROR: $1"
    exit 1
}

# Check if cargo is already in path
check_cargo && return

# Check if rustup-init installed .cargo/env is available
if [[ -r $HOME/.cargo/env ]]; then
    . "$HOME/.cargo/env"
    check_cargo 'No cargo after sourcing ~/.cargo/env' && return
fi

# Check for brew or search for default install
if ! command -v brew /dev/null && [[ -x /opt/homebrew/bin/brew ]]; then
    eval "$(/opt/homebrew/bin/brew shellenv)"
fi

# Check if brew installed rustup is available
if command -v brew > /dev/null && brew --prefix rustup > /dev/null 2>&1; then
    export PATH="$(brew --prefix rustup)/bin:$PATH"
    check_cargo 'No cargo after adding $(brew --prefix rustup)/bin to $PATH' && return
fi

fail 'Cannot find cargo'
