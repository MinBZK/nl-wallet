import html
import re
from pathlib import Path
from typing import Any

from docutils import nodes
from sphinx.application import Sphinx
from sphinx.util import logging as sphinx_logging

logger = sphinx_logging.getLogger(__name__)

_LTC_REF_RE = re.compile(r'\[LTC(\d+)', re.IGNORECASE)
_TITLE_RE = re.compile(r'^#\s+(.+)$', re.MULTILINE)


def _scan_test_files(app: Sphinx) -> dict[str, list[dict[str, Any]]]:
    """Scan test directories for LTC references.

    Returns a dict mapping LTC IDs to a list of implementation locations.
    """
    project_root = Path(app.config.ltc_project_root)
    test_dirs = app.config.ltc_test_dirs
    file_patterns = app.config.ltc_file_patterns
    ltc_re = re.compile(app.config.ltc_pattern, re.IGNORECASE)

    implementations: dict[str, list[dict[str, Any]]] = {}

    for test_dir in test_dirs:
        scan_root = project_root / test_dir
        if not scan_root.is_dir():
            logger.warning(f'[ltc_linker] Test directory does not exist: {scan_root}')
            continue

        for pattern in file_patterns:
            for filepath in scan_root.glob(pattern):
                if not filepath.is_file():
                    continue

                try:
                    content = filepath.read_text(encoding='utf-8', errors='replace')
                except OSError as e:
                    logger.warning(f'[ltc_linker] Could not read {filepath}: {e}')
                    continue

                rel_path = str(filepath.relative_to(project_root))

                for line_number, line in enumerate(content.splitlines(), start=1):
                    for match in ltc_re.finditer(line):
                        ltc_id = match.group('id')
                        if ltc_id not in implementations:
                            implementations[ltc_id] = []

                        implementations[ltc_id].append({
                            'file_path': rel_path,
                            'line_number': line_number,
                            'test_dir': test_dir,
                        })

    for ltc_id in implementations:
        implementations[ltc_id].sort(key=lambda e: (e['test_dir'], e['file_path']))

    return implementations


def _scan_design_files(app: Sphinx) -> dict[str, list[dict[str, Any]]]:
    """Scan functional design files for LTC references in overview tables.

    Returns a dict mapping LTC IDs to a list of UC/PF documents that reference them.
    """
    src_dir = Path(app.srcdir)
    covers_dirs = app.config.ltc_covers_dirs

    covers: dict[str, list[dict[str, Any]]] = {}

    for covers_dir in covers_dirs:
        scan_dir = src_dir / covers_dir
        if not scan_dir.is_dir():
            logger.warning(f'[ltc_linker] Covers directory does not exist: {scan_dir}')
            continue

        for filepath in sorted(scan_dir.glob('**/*.md')):
            try:
                content = filepath.read_text(encoding='utf-8', errors='replace')
            except OSError as e:
                logger.warning(f'[ltc_linker] Could not read {filepath}: {e}')
                continue

            title_match = _TITLE_RE.search(content)
            title = title_match.group(1) if title_match else filepath.stem
            docname = str(filepath.relative_to(src_dir).with_suffix(''))

            for line in content.splitlines():
                if '**logical test cases**' not in line.lower():
                    continue
                for m in _LTC_REF_RE.finditer(line):
                    ltc_id = m.group(1)
                    if ltc_id not in covers:
                        covers[ltc_id] = []
                    covers[ltc_id].append({
                        'title': title,
                        'docname': docname,
                    })

    return covers


def _has_manual_marker(section: nodes.section) -> bool:
    """Return True if the section contains a '% manual' comment."""
    for comment in section.findall(nodes.comment):
        if comment.astext().strip().lower() == 'manual':
            return True
    return False


def _find_transition(section: nodes.section) -> tuple[nodes.Node, int] | None:
    """Find the first transition (---) among the direct children of a section.

    Returns (parent_node, index) or None.
    """
    for i, child in enumerate(section.children):
        if isinstance(child, nodes.transition):
            return section, i
    return None


def _build_impl_nodes(
    implementations: list[dict[str, Any]],
    repo_url: str,
    repo_ref: str,
    is_manual: bool,
) -> nodes.raw:
    """Build the Implementations block for a single LTC."""
    parts = ['<details class="ltc-implementations"><summary>Implementations</summary><ul>']

    if is_manual:
        parts.append('<li><em>Manual test</em></li>')

    for impl in implementations:
        file_path = impl['file_path']
        line_number = impl['line_number']
        display = html.escape(f'{file_path}:{line_number}')
        url = html.escape(f'{repo_url.rstrip("/")}/{repo_ref}/{file_path}#L{line_number}')
        parts.append(f'<li><a href="{url}"><code>{display}</code></a></li>')

    parts.append('</ul></details>')
    return nodes.raw('', ''.join(parts), format='html')


def _build_covers_nodes(
    covers: list[dict[str, Any]],
    app: Sphinx,
    current_docname: str,
) -> nodes.raw:
    """Build the Covers by block for a single LTC."""
    parts = ['<details class="ltc-covers"><summary>Covered Use Case</summary><ul>']

    for cover in covers:
        title = cover['title']
        url = app.builder.get_relative_uri(current_docname, cover['docname'])
        parts.append(f'<li><a href="{url}"><code>{title}</code></a></li>')

    parts.append('</ul></details>')
    return nodes.raw('', ''.join(parts), format='html')


def _get_repo_ref(app: Sphinx) -> str:
    """Return the git ref (branch or tag) for the current build.

    sphinx-multiversion builds each version from its own conf.py, so
    app.config.release reflects that version's release string.
    Dev builds (ending in '-dev') point to main; tagged releases use 'v{release}'.
    """
    release = app.config.release
    if release.endswith('-dev'):
        return 'main'
    return f'v{release}'


def _find_ltc_section_id(title_text: str) -> str | None:
    """Extract the LTC number from a section title like 'LTC42'."""
    m = re.match(r'^LTC(\d+)$', title_text.strip(), re.IGNORECASE)
    if m:
        return m.group(1)
    return None


def process_ltc_links(app: Sphinx, doctree: nodes.document, docname: str) -> None:
    """Event handler for 'doctree-resolved'.

    Walks the doctree looking for sections whose title matches 'LTC<N>'.
    For each match, inserts an Implementations block and a Covered by block.
    """
    if docname != app.config.ltc_source_page:
        return

    implementations = _scan_test_files(app)
    covers = _scan_design_files(app)
    repo_url = app.config.ltc_repo_url
    repo_ref = _get_repo_ref(app)

    no_impl: list[str] = []
    no_covers: list[str] = []

    for section in doctree.findall(nodes.section):
        title_node = section.next_node(nodes.title)
        if title_node is None:
            continue

        ltc_id = _find_ltc_section_id(title_node.astext())
        if ltc_id is None:
            continue

        is_manual = _has_manual_marker(section)
        code_impls = implementations.get(ltc_id, [])
        design_covers = covers.get(ltc_id, [])

        if not code_impls and not is_manual:
            no_impl.append(ltc_id)
        if not design_covers:
            no_covers.append(ltc_id)

        if not code_impls and not is_manual and not design_covers:
            continue

        impl_nodes = _build_impl_nodes(code_impls, repo_url, repo_ref, is_manual)
        covers_nodes = _build_covers_nodes(design_covers, app, docname)

        location = _find_transition(section)
        if location is not None:
            parent, idx = location
            parent.children.insert(idx, covers_nodes)
            parent.children.insert(idx, impl_nodes)
        else:
            section += impl_nodes
            section += covers_nodes

    def _fmt(ids: list[str]) -> str:
        return ', '.join(f'LTC{i}' for i in sorted(ids, key=int))

    if no_impl:
        logger.warning(f'[ltc_linker] LTCs without implementation: {_fmt(no_impl)}')
    if no_covers:
        logger.warning(f'[ltc_linker] LTCs not covering any UC/PF: {_fmt(no_covers)}')


def setup(app: Sphinx) -> dict[str, Any]:
    app.add_config_value('ltc_test_dirs', default=[], rebuild='html')
    app.add_config_value('ltc_file_patterns', default=['**/*.kt', '**/*.dart', '**/*.rs'], rebuild='html')
    app.add_config_value('ltc_repo_url', default='', rebuild='html')
    app.add_config_value('ltc_project_root', default=None, rebuild='html')
    app.add_config_value('ltc_source_page', default='logical-test-cases', rebuild='html')
    app.add_config_value('ltc_pattern', default=r'LTC(?P<id>\d+)', rebuild='html')
    app.add_config_value('ltc_covers_dirs', default=['functional-design/use-cases', 'functional-design/partial-flows'], rebuild='html')

    app.connect('doctree-resolved', process_ltc_links)

    return {
        'version': '1.0.0',
        'parallel_read_safe': True,
        'parallel_write_safe': True,
    }
