import 'package:flutter/material.dart';

/// Model that contains all info to render a single policy entry
class PolicyEntry {
  final IconData? icon;
  final TextSpan title;
  final TextSpan description;

  const PolicyEntry({
    this.icon,
    required this.title,
    required this.description,
  });
}
