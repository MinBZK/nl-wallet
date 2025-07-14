---
orphan: true
---
# NL Wallet Documentation

This folder contains the contents and configuration to create our project documentation with Sphinx.

Content is written in Markdown and in the CI this is build as a static html site and deployed to [GitHub pages](https://minbzk.github.io/nl-wallet/) 

In Sphinx the markdown files are parsed with `myst-parser`. There are minor differences in how `myst-parser` parses the Markdown files in comparison to standard Markdown parsers.

For instance a table of contents should be written as described [here](https://sphinx-doc-zh.readthedocs.io/en/latest/markup/toctree.html) and a Mermaid diagram as described [here](https://github.com/mgaitan/sphinxcontrib-mermaid#markdown-support).

When transferring the documentation from pure Markdown to Sphinx with Markdown these were the only 2 (small) differences. If something behaves unexpected please refer to the [Sphinx Documentation](https://www.sphinx-doc.org/en/master/index.html) or the [myst-parser Documentation](https://myst-parser.readthedocs.io/en/latest/index.html).

Documentation is build in every MR pipeline and this jobs verifies if broken links, both internal and external, are not broken and if there are any Markdown Syntax errors.

To verify this locally run the following commands from within the documentation directory:

```sh
python3 -m venv venv 
source venv/bin/activate
pip3 install -r requirements.txt
make clean
make linkcheck
sphinx-build -b html -W . _build/html
```