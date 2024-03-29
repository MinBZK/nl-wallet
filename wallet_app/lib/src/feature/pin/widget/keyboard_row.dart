import 'package:flutter/material.dart';

class KeyboardRow extends StatelessWidget {
  final List<Widget> children;

  const KeyboardRow({this.children = const <Widget>[], super.key});

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: children,
      ),
    );
  }
}
