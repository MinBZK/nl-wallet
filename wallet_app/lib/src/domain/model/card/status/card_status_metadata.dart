import 'package:flutter/material.dart';

/// Metadata class that holds the formatted UI data for a card status.
class CardStatusMetadata {
  final String text;
  final Color textColor;
  final IconData? icon;
  final Color? iconColor;
  final Color? backgroundColor;

  CardStatusMetadata({
    required this.text,
    required this.textColor,
    this.icon,
    this.iconColor,
    this.backgroundColor,
  });
}
