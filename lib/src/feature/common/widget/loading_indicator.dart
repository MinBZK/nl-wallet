import 'package:flutter/material.dart';

import '../../../../environment.dart';

class LoadingIndicator extends StatelessWidget {
  const LoadingIndicator({super.key});

  @override
  Widget build(BuildContext context) {
    return CircularProgressIndicator(value: Environment.isTest ? 0.5 : null);
  }
}
