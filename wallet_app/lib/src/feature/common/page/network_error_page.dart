import 'package:flutter/material.dart';

import '../../error/error_page.dart';

class NetworkErrorPage extends StatelessWidget {
  final VoidCallback onStopPressed;
  final bool hasInternet;

  const NetworkErrorPage({
    required this.onStopPressed,
    required this.hasInternet,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    if (hasInternet) {
      return ErrorPage.network(
        context,
        style: ErrorCtaStyle.close,
        onPrimaryActionPressed: onStopPressed,
      );
    } else {
      return ErrorPage.noInternet(
        context,
        style: ErrorCtaStyle.close,
        onPrimaryActionPressed: onStopPressed,
      );
    }
  }
}
