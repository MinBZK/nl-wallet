import html
import re
from pathlib import Path
from typing import Any

from docutils import nodes
from sphinx.application import Sphinx
from sphinx.errors import ExtensionError
from sphinx.util import logging as sphinx_logging

logger = sphinx_logging.getLogger(__name__)

_TITLE_RE = re.compile(r'^#\s+(.+)$')
_ERROR_REF_RE = re.compile(r'errors\.md#([\w-]+)')
_POSSIBLE_ERRORS_RE = re.compile(r'\*\*Possible errors\*\*')
_ERROR_ANCHOR_RE = re.compile(r'\(#([\w-]+)\)')


def _slug(text: str) -> str:
    text = re.sub(r'[^\w\s-]', '', text.strip().lower())
    return re.sub(r'\s+', '-', text)


def _read_error_slugs(source_file: Path) -> set[str]:
    """Parse errors.md and return error slugs from the index list"""
    try:
        content = source_file.read_text(encoding='utf-8', errors='replace')
    except OSError as e:
        raise ExtensionError(f'[error_linker] Could not read source file {source_file}: {e}') from e

    slugs = {m.group(1) for line in content.splitlines() if (m := _ERROR_ANCHOR_RE.search(line))}
    if not slugs:
        raise ExtensionError(f'[error_linker] No error anchors found in {source_file}')

    logger.info(f'[error_linker] Found {len(slugs)} Error(s) in {source_file.name}: {", ".join(slugs)}')
    return slugs


def _scan_uc_pf_files(app: Sphinx) -> dict[str, list[dict[str, Any]]]:
    """Scan use-case and partial-flow files for error references in 'Possible errors' rows.

    Returns a dict mapping error slug to a list of {title, docname} dicts.
    """
    src_dir = Path(app.srcdir)
    scan_dirs = app.config.error_linker_scan_dirs

    error_refs: dict[str, list[dict[str, Any]]] = {}

    for scan_dir in scan_dirs:
        dir_path = src_dir / scan_dir
        if not dir_path.is_dir():
            logger.warning(f'[error_linker] Directory does not exist: {dir_path}')
            continue

        for filepath in sorted(dir_path.glob('**/*.md')):
            try:
                content = filepath.read_text(encoding='utf-8', errors='replace')
            except OSError as e:
                logger.warning(f'[error_linker] Could not read {filepath}: {e}')
                continue
            title_match = _TITLE_RE.search(content)
            title = title_match.group(1) if title_match else None
            docname = str(filepath.relative_to(src_dir).with_suffix(''))

            for line in content.splitlines():
                if not _POSSIBLE_ERRORS_RE.search(line):
                    continue
                for m in _ERROR_REF_RE.finditer(line):
                    error_slug = m.group(1)
                    if error_slug not in error_refs:
                        error_refs[error_slug] = []
                    entry = {'title': title, 'docname': docname}
                    error_refs[error_slug].append(entry)

    for slug, refs in error_refs.items():
        logger.info(f'[error_linker] Error "{slug}" referenced by {len(refs)} document(s)')

    return error_refs


def _build_referenced_by_nodes(
    refs: list[dict[str, Any]],
    app: Sphinx,
    current_docname: str,
) -> nodes.raw:
    """Build the 'Occurs in' block for a single error section."""
    parts = ['<details class="error-references"><summary>Occurs in</summary><ul>']

    for ref in refs:
        title = html.escape(ref['title'] or ref['docname'])
        url = app.builder.get_relative_uri(current_docname, ref['docname'])
        parts.append(f'<li><a href="{url}">{title}</a></li>')

    parts.append('</ul></details>')
    return nodes.raw('', ''.join(parts), format='html')


def process_error_links(app: Sphinx, doctree: nodes.document, docname: str) -> None:
    """Event handler for 'doctree-resolved'.

    Walks the errors.md doctree and injects a 'Occurs in' block into each error
    section. Raises ExtensionError if any configured error has no UC/PF references.
    """
    if docname != app.config.error_linker_source_page:
        return

    logger.info(f'[error_linker] Processing {docname}')

    error_refs = _scan_uc_pf_files(app)
    source_file = Path(app.srcdir) / (app.config.error_linker_source_page + '.md')
    error_slugs = _read_error_slugs(source_file)

    no_refs: list[str] = []

    for section in doctree.findall(nodes.section):
        title_node = section.next_node(nodes.title)
        if title_node is None:
            continue

        title_text = title_node.astext()
        section_slug = _slug(title_text)

        if section_slug not in error_slugs:
            continue

        refs = error_refs.get(section_slug, [])
        if not refs:
            no_refs.append(title_text)
            logger.warning(f'[error_linker] Error "{title_text}" ({section_slug}) has no use case or partial flow references')
            continue

        logger.debug(f'[error_linker] Injecting {len(refs)} reference(s) into section "{title_text}"')
        section += _build_referenced_by_nodes(refs, app, docname)

    if no_refs:
        formatted = ', '.join(f'"{e}"' for e in sorted(no_refs))
        raise ExtensionError(
            f'[error_linker] The following errors in "{docname}" have no use case or partial flow references: {formatted}'
        )


def setup(app: Sphinx) -> dict[str, Any]:
    app.add_config_value(
        'error_linker_source_page',
        default='functional-design/errors',
        rebuild='html',
    )
    app.add_config_value(
        'error_linker_scan_dirs',
        default=['functional-design/use-cases', 'functional-design/partial-flows'],
        rebuild='html',
    )
    app.connect('doctree-resolved', process_error_links)

    return {
        'version': '1.0.0',
        'parallel_read_safe': True,
        'parallel_write_safe': True,
    }
