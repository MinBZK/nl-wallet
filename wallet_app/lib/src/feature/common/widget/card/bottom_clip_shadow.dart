import 'package:flutter/material.dart';

import 'wallet_card_item.dart';

/// Draws an overlay at the bottom of this widget, which curves
/// upwards based on the provided [radius]. Used to draw the
/// shadow at the bottom of a [WalletCardItem].
class BottomClipShadow extends StatelessWidget {
  final double radius;
  final int bottomOffset;
  final Color shadowColor;

  const BottomClipShadow({
    required this.radius,
    this.bottomOffset = 1,
    this.shadowColor = const Color(0x0D000000),
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ClipPath(
      clipper: BottomArcShadowClipper(radius),
      child: Container(color: shadowColor),
    );
  }
}

class BottomArcShadowClipper extends CustomClipper<Path> {
  final double radius;
  final double bottomOffset;

  @override
  Path getClip(Size size) {
    final Path path = Path();

    // Start on the left size, at the height of the requested radius
    path.lineTo(0, size.height - radius);

    // Create an arc from the left to the [bottomOffset]
    path.arcToPoint(
      Offset(radius, size.height - bottomOffset),
      radius: Radius.circular(radius),
      clockwise: false,
    );

    // Sweep from the end of the left arc, to the start of the right arc
    path.lineTo(size.width - radius, size.height - bottomOffset);

    // Arc to the right edge, ending at the radius's height
    path.arcToPoint(
      Offset(size.width, size.height - radius),
      radius: Radius.circular(radius),
      clockwise: false,
    );

    // Move to the bottom right
    path.lineTo(size.width, size.height);

    // Line to the bottom bottom left
    path.lineTo(0, size.height);

    // Close the path (automatically returns to the start point)
    path.close();

    return path;
  }

  @override
  bool shouldReclip(CustomClipper<Path> oldClipper) => true;

  BottomArcShadowClipper(this.radius, {this.bottomOffset = 1});
}
