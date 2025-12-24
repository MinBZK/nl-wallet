# NL Wallet Documentation

This directory contains the Sphinx-based documentation for the NL Wallet
project.

## Prerequisites

- Python 3
- pip3

## Building the Documentation

### Initial Setup

1. Navigate to the documentation directory:

    ```bash
    cd wallet_docs
    ```

2. Create a Python virtual environment:

    ```bash
    python3 -m venv .venv
    ```

3. Activate the virtual environment:

    ```bash
    source .venv/bin/activate
    ```

4. Install dependencies:
    ```bash
    pip3 install -r requirements.txt
    ```

### Building

Use `sphinx-multiversion` to build documentation for all versions (alternatively
you can use `make html` to build for the current branch/version):

```bash
make linkcheck
sphinx-multiversion . _build/html -W
```

### Viewing the Documentation

After building, open the generated HTML files in your browser:

```bash
open _build/html/index.html
```

## Available Make Targets

- `make html` - Build HTML documentation
- `make linkcheck` - Check for broken links
- `make clean` - Remove built documentation

## Live Preview with Auto-Rebuild

For development, you can use `sphinx-autobuild` (included in requirements.txt):

```bash
sphinx-autobuild . _build/html
```

This will start a local web server and automatically rebuild the documentation
when files change.

## Notes

- The `-W` flag treats warnings as errors, ensuring documentation quality
- `sphinx-multiversion` is used in CI/CD to build documentation for multiple Git
  branches/tags
- Link checking is performed before each build to catch broken external
  references
