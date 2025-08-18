# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

author = 'NL Wallet'
project = author
copyright = f"2025, {author}"
version = '0.4.0-dev'
release = version

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'myst_parser',
    'sphinx_multiversion',
    'sphinx.builders.linkcheck',
    'sphinxcontrib.mermaid',
]

source_suffix = {
    '.md': 'markdown',
}

templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store', '.venv', '_venv', 'venv']

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_logo = '_static/img/non-free/wallet.svg'
html_css_files = ['css/custom.css']
html_js_files = ['js/custom.js']
html_static_path = ['_static']
html_show_sphinx = False
html_theme = 'sphinx_rtd_theme'

html_theme_options = {
    'logo_only': False,
    'prev_next_buttons_location': 'bottom',
    'vcs_pageview_mode': '',
    'flyout_display': 'hidden',
    'version_selector': True,
    'language_selector': False,
    'collapse_navigation': False,
    'sticky_navigation': False,
    'navigation_depth': 2,
    'includehidden': False,
    'titles_only': True,
    'style_external_links': True,
    'style_nav_header_background': '#383EDE',
}

myst_html_meta = {}
myst_heading_anchors = 5

myst_enable_extensions = [
    "deflist",
    "colon_fence",
    "attrs_block",
    "html_admonition",
    "html_image",
]

myst_substitutions = {
    "project_name": "NL Wallet",
}

smv_tag_whitelist = r'^v\d+\.\d+\.\d+$'
smv_branch_whitelist = r'^main$'

linkcheck_report_timeouts_as_broken = False

linkcheck_ignore = [
    r'https://www\.iso\.org/obp/ui/en/#iso:std:iso-iec:18013:-5:ed-1:v1:en',
    r'https://github\.com/mgaitan/sphinxcontrib-mermaid#markdown-support',
]

nitpicky = True
