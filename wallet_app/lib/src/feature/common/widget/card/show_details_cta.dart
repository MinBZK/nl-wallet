import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

const _kButtonHeight = 24.0;

class ShowDetailsCta extends StatelessWidget {
  final Text text;
  final Color? textColor;
  final VoidCallback? onPressed;
  final WidgetStatesController? statesController;

  const ShowDetailsCta({
    required this.text,
    this.textColor,
    this.onPressed,
    this.statesController,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TextButton.icon(
      icon: const Icon(Icons.arrow_forward),
      iconAlignment: IconAlignment.end,
      label: text,
      onPressed: onPressed,
      statesController: statesController,
      style: _resolveButtonStyle(context),
    );
  }

  ButtonStyle _resolveButtonStyle(BuildContext context) {
    return context.theme.textButtonTheme.style!.copyWith(
      backgroundColor: const WidgetStatePropertyAll(
        Colors.transparent,
      ),
      foregroundColor: WidgetStatePropertyAll(
        textColor,
      ),
      iconColor: WidgetStatePropertyAll(
        textColor,
      ),
      minimumSize: const WidgetStatePropertyAll(
        Size(0, _kButtonHeight),
      ),
      padding: const WidgetStatePropertyAll(
        EdgeInsets.zero,
      ),
      tapTargetSize: MaterialTapTargetSize.shrinkWrap,
      shape: const WidgetStatePropertyAll(
        RoundedRectangleBorder(
          borderRadius: BorderRadius.zero,
        ),
      ),
      side: const WidgetStatePropertyAll(BorderSide.none),
    );
  }
}
