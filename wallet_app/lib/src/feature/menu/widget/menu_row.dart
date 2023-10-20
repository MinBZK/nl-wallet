import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

const _kMinHeight = 56.0;

class MenuRow extends StatelessWidget {
  final IconData? icon;
  final String label;
  final VoidCallback onTap;

  const MenuRow({
    Key? key,
    this.icon,
    required this.label,
    required this.onTap,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      child: ConstrainedBox(
        constraints: const BoxConstraints(minHeight: _kMinHeight),
        child: InkWell(
          onTap: onTap,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              _buildLeading(context),
              Expanded(
                child: Text(
                  label,
                  style: context.textTheme.titleMedium,
                ),
              ),
              const SizedBox(
                width: _kMinHeight,
                child: Center(
                  child: Icon(Icons.chevron_right),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildLeading(BuildContext context) {
    if (icon == null) return const SizedBox(width: 16);
    return SizedBox(
      width: _kMinHeight,
      child: Center(
        child: Icon(
          icon,
          color: context.colorScheme.onSurface,
        ),
      ),
    );
  }
}
