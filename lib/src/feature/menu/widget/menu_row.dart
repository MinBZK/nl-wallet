import 'package:flutter/material.dart';

const _kMinHeight = 56.0;

class MenuRow extends StatelessWidget {
  final IconData? icon;
  final String label;
  final VoidCallback onTap;

  const MenuRow({Key? key, this.icon, required this.label, required this.onTap}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ConstrainedBox(
      constraints: const BoxConstraints(minHeight: _kMinHeight),
      child: InkWell(
        onTap: onTap,
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            _buildLeading(),
            Expanded(
              child: Text(
                label,
                style: Theme.of(context).textTheme.subtitle1,
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
    );
  }

  Widget _buildLeading() {
    if (icon == null) return const SizedBox(width: 16);
    return SizedBox(
      width: _kMinHeight,
      child: Center(child: Icon(icon)),
    );
  }
}
