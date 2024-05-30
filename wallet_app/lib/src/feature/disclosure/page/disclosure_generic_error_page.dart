import 'package:flutter/material.dart';

import '../../error/error_page.dart';

class DisclosureGenericErrorPage extends StatelessWidget {
  final VoidCallback onStopPressed;

  const DisclosureGenericErrorPage({
    required this.onStopPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ErrorPage.generic(
      context,
      style: ErrorCtaStyle.close,
      onPrimaryActionPressed: onStopPressed,
    );
  }
}
