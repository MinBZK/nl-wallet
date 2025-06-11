# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'NL Wallet'
copyright = '2025, NL Wallet'
author = 'NL Wallet'
release = '0.1'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'myst_parser',
    'sphinxcontrib.mermaid',
    'sphinx.builders.linkcheck',
    'sphinx_multiversion',
]

source_suffix = {
    '.md': 'markdown',
}

templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store', 'venv']

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_rtd_theme'

html_static_path = ['_static']

html_css_files = [
    'css/custom.css',
]

html_theme_options = {
    'logo_only': False,
    'prev_next_buttons_location': 'bottom',
    'vcs_pageview_mode': '',
    'flyout_display': 'hidden',
    'version_selector': True,
    'language_selector': False,
    'collapse_navigation': False,
    'sticky_navigation': True,
    'navigation_depth': 4,
    'includehidden': True,
    'titles_only': False,
    'style_external_links': True,
    'style_nav_header_background': '#383EDE',
}

myst_html_meta = {}

html_show_sphinx = False

html_logo = '_static/wallet.png'

html_extra_path = ['images']

myst_enable_extensions = [
    "deflist",
    "colon_fence",
    "attrs_block",
    "html_admonition",
    "html_image",
]

myst_heading_anchors = 5

myst_substitutions = {
    "project_name": "NL Wallet",
}

smv_tag_whitelist = r'^v\d+\.\d+\.\d+$'
smv_branch_whitelist = r'^main$'

linkcheck_ignore = [
    r'https://www\.iso\.org/obp/ui/en/#iso:std:iso-iec:18013:-5:ed-1:v1:en',
    r'https://github\.com/mgaitan/sphinxcontrib-mermaid#markdown-support',
]

nitpicky = True
