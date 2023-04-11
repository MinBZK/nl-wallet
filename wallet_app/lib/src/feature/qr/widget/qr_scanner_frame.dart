import 'package:flutter/material.dart';

class QrScannerFrame extends StatelessWidget {
  final Widget child;

  const QrScannerFrame({required this.child, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16.0),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(8),
        child: AspectRatio(
          aspectRatio: 1.0,
          child: child,
        ),
      ),
    );
  }
}
