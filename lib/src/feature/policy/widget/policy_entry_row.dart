import 'package:flutter/material.dart';

class PolicyEntryRow extends StatelessWidget {
  final IconData? icon;
  final Widget title;
  final Widget description;

  const PolicyEntryRow({
    this.icon,
    required this.title,
    required this.description,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          icon == null ? const SizedBox.shrink() : Icon(icon, size: 24),
          SizedBox(width: icon == null ? 0 : 16),
          Expanded(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              mainAxisAlignment: MainAxisAlignment.start,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                ConstrainedBox(
                  constraints: const BoxConstraints(minHeight: 24),
                  child: DefaultTextStyle(
                    style: Theme.of(context).textTheme.subtitle1!,
                    child: title,
                  ),
                ),
                const SizedBox(height: 8),
                DefaultTextStyle(
                  style: Theme.of(context).textTheme.bodyText1!,
                  child: description,
                ),
              ],
            ),
          )
        ],
      ),
    );
  }
}
