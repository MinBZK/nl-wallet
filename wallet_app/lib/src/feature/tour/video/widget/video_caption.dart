import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

class VideoCaption extends StatelessWidget {
  final String caption;

  const VideoCaption({
    required this.caption,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    if (caption.isEmpty) return const SizedBox.shrink();
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 0, horizontal: 40),
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: const Color(0xB2191C1B),
            borderRadius: BorderRadius.circular(4),
          ),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 8),
            child: Text(
              caption,
              textAlign: TextAlign.center,
              style: context.textTheme.bodyLarge?.copyWith(
                color: Colors.white,
                shadows: [
                  const Shadow(
                    color: Colors.black,
                    offset: Offset(0, 2),
                    blurRadius: 8,
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
