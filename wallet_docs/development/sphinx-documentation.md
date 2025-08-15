# Sphinx Documentation

We create our documentation site with Sphinx.

Content is written in Markdown and in the CI this is built as a static html site
and deployed to [GitHub pages](https://minbzk.github.io/nl-wallet/)

In Sphinx, the markdown files are parsed with `myst-parser`. There are minor
differences in how `myst-parser` parses the Markdown files in comparison to
other Markdown parsers; for instance, a table of contents should be written as
described
[here](https://sphinx-doc-zh.readthedocs.io/en/latest/markup/toctree.html) and a
Mermaid diagram should be written as described
[here](https://github.com/mgaitan/sphinxcontrib-mermaid#markdown-support).

When transferring the documentation from pure Markdown to Sphinx with Markdown
these were the only 2 (small) differences. If something behaves unexpectedly,
please refer to the [Sphinx
documentation](https://www.sphinx-doc.org/en/master/index.html) or the
[myst-parser
documentation](https://myst-parser.readthedocs.io/en/latest/index.html).

Documentation is built in every MR pipeline and this job verifies if there are
any broken links, both internal and external, and if there are any Markdown
syntax errors.

To verify this locally, run the following commands from within the documentation
directory:

```sh
python3 -m venv .venv
source .venv/bin/activate
pip3 install -r requirements.txt
make clean
make linkcheck
sphinx-build -b html -W . _build/html
# Or if you're working on the documentation and want to localhost:8000 it:
sphinx-autobuild -b html -W . _build/html
```
