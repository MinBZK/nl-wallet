import 'package:flutter/cupertino.dart';

class SliverSizedBox extends StatelessWidget {
  final double? width, height;
  final Widget? child;

  const SliverSizedBox({
    this.width,
    this.height,
    this.child,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SliverToBoxAdapter(
      child: SizedBox(
        width: width,
        height: height,
        child: child,
      ),
    );
  }
}
