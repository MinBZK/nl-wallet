import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../common/widget/placeholder_screen.dart';

class IssuanceScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) id when opening the IssuanceScreen');
    }
  }

  const IssuanceScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return const PlaceholderScreen(title: 'Issuance');
  }
}
