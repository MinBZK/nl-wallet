import 'package:flutter/widgets.dart';

extension BuildContextExtension on BuildContext {
  bool get isLandscape => MediaQuery.of(this).orientation == Orientation.landscape;
}
