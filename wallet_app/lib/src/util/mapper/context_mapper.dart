import 'package:flutter/widgets.dart';

abstract class ContextMapper<I, O> {
  O map(BuildContext context, I input);
}
