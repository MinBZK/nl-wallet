import 'package:flutter/material.dart';

/// Model that contains all info to render a single policy entry
class PolicyEntry {
  final IconData? icon;
  final TextSpan title;
  final TextSpan description;
  final String? titleSemanticsLabel;
  final String? descriptionSemanticsLabel;
  final VoidCallback? semanticOnTap;
  final String? semanticOnTapHint;

  const PolicyEntry({
    this.icon,
    required this.title,
    required this.description,
    this.titleSemanticsLabel,
    this.descriptionSemanticsLabel,
    this.semanticOnTap,
    this.semanticOnTapHint,
  });
}
