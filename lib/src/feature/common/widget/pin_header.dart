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
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 32),
      child: _buildHeader(context),
    );
  }

  Widget _buildHeader(BuildContext context) {
    final textColor = hasError ? Theme.of(context).errorColor : null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisAlignment: MainAxisAlignment.start,
      children: [
        Text(
          title,
          style: Theme.of(context).textTheme.headline2?.copyWith(color: textColor),
        ),
        const SizedBox(height: 8),
        Text(
          description,
          style: Theme.of(context).textTheme.bodyText1?.copyWith(color: textColor),
        ),
      ],
    );
  }
}
