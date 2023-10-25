import 'package:flutter/widgets.dart';

abstract class ContextMapper<I, O> {
  O map(BuildContext context, I input);

  List<O> mapList(BuildContext context, List<I> input) => input.map((e) => map(context, e)).toList();
}
