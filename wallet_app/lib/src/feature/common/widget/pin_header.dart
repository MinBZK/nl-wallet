import 'package:flutter/material.dart';

class PinHeader extends StatelessWidget {
  final String title;
  final String description;
  final bool hasError;

  const PinHeader({
    Key? key,
    required this.title,
    required this.description,
    required this.hasError,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      child: _buildHeader(context),
    );
  }

  Widget _buildHeader(BuildContext context) {
    final textColor = hasError ? Theme.of(context).colorScheme.error : null;
    return Column(
      crossAxisAlignment: _resolveCrossAxisAlignment(context),
      mainAxisAlignment: MainAxisAlignment.start,
      children: [
        Text(
          title,
          style: Theme.of(context).textTheme.displayMedium?.copyWith(color: textColor),
          textAlign: _resolveTextAlignment(context),
        ),
        const SizedBox(height: 8),
        Text(
          description,
          style: Theme.of(context).textTheme.bodyLarge?.copyWith(color: textColor),
          textAlign: _resolveTextAlignment(context),
        ),
      ],
    );
  }

  CrossAxisAlignment _resolveCrossAxisAlignment(BuildContext context) {
    final orientation = MediaQuery.of(context).orientation;
    switch (orientation) {
      case Orientation.portrait:
        return CrossAxisAlignment.start;
      case Orientation.landscape:
        return CrossAxisAlignment.center;
    }
  }

  TextAlign _resolveTextAlignment(BuildContext context) {
    final orientation = MediaQuery.of(context).orientation;
    switch (orientation) {
      case Orientation.portrait:
        return TextAlign.start;
      case Orientation.landscape:
        return TextAlign.center;
    }
  }
}
