import 'package:flutter/cupertino.dart';

class SliverSizedBox extends StatelessWidget {
  final double? width, height;

  const SliverSizedBox({this.width, this.height, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SliverToBoxAdapter(
      child: SizedBox(
        width: width,
        height: height,
      ),
    );
  }
}
