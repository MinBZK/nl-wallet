import 'package:flutter/material.dart';

class BottomSheetDragHandle extends StatelessWidget {
  const BottomSheetDragHandle({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 32,
      height: 4,
      decoration: BoxDecoration(
        color: Theme.of(context).dividerColor,
        borderRadius: BorderRadius.circular(4),
      ),
    );
  }
}
