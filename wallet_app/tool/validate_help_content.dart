import 'dart:io';

import 'package:wallet/src/util/help_content_validator.dart';

const _helpDir = 'assets/non-free/markdown/help';

/// CLI wrapper around [validateHelpContent]. Run from the `wallet_app` dir:
///
///   fvm dart run tool/validate_help_content.dart
///
/// Exits 0 when clean, 1 on any issue. Writes a human-readable report grouped
/// by file.
void main(List<String> args) {
  if (!Directory(_helpDir).existsSync()) {
    stderr.writeln(
      'Could not find $_helpDir. Run this script from the wallet_app/ directory.',
    );
    exit(2);
  }

  stdout.writeln('Validating help content in $_helpDir …\n');

  final result = validateHelpContent(helpDir: _helpDir);
  _printReport(result);

  if (result.hasIssues) exit(1);
}

void _printReport(ValidationResult result) {
  final colored = stdout.supportsAnsiEscapes;
  final byFile = <String, List<ContentIssue>>{};
  for (final issue in result.issues) {
    byFile.putIfAbsent(issue.file, () => []).add(issue);
  }

  final sortedFiles = byFile.keys.toList()..sort();
  for (final file in sortedFiles) {
    stdout.writeln('  $file');
    for (final issue in byFile[file]!) {
      final label = colored ? '\x1B[31merror\x1B[0m' : 'error';
      stdout.writeln('    $label  ${issue.message}');
    }
    stdout.writeln();
  }

  if (result.issues.isEmpty) {
    stdout.writeln(colored ? '\x1B[32m✓ Help content is valid.\x1B[0m' : '✓ Help content is valid.');
  } else {
    final count = result.issues.length;
    stdout.writeln(
      'Result: $count ${_plural(count, 'error')} across ${byFile.length} ${_plural(byFile.length, 'file')}.',
    );
  }
}

String _plural(int count, String word) => count == 1 ? word : '${word}s';
