import 'package:flutter/material.dart';

const double _kSuccessIconSize = 56;

class SuccessIcon extends StatelessWidget {
  const SuccessIcon({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      width: _kSuccessIconSize,
      height: _kSuccessIconSize,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: Theme.of(context).colorScheme.primary,
      ),
      child: const Icon(
        Icons.check,
        color: Colors.white,
      ),
    );
  }
}
