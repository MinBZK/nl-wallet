import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../widget/button/confirm/confirm_button.dart';
import '../widget/button/confirm/confirm_buttons.dart';

class ConfirmActionSheet extends StatelessWidget {
  final VoidCallback? onCancelPressed;
  final VoidCallback? onConfirmPressed;
  final String title;
  final String description;
  final String cancelButtonText;
  final IconData? cancelIcon;
  final String confirmButtonText;
  final IconData? confirmIcon;
  final Color? confirmButtonColor;
  final Widget? extraContent;

  const ConfirmActionSheet({
    this.onCancelPressed,
    this.onConfirmPressed,
    this.confirmButtonColor,
    required this.title,
    required this.description,
    required this.cancelButtonText,
    this.cancelIcon,
    required this.confirmButtonText,
    this.confirmIcon,
    this.extraContent,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Theme(
      data: context.theme.copyWith(elevatedButtonTheme: buttonTheme(context)),
      child: SafeArea(
        minimum: const EdgeInsets.symmetric(vertical: 24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            MergeSemantics(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: Text(
                      title,
                      style: context.textTheme.displayMedium,
                      textAlign: TextAlign.start,
                    ),
                  ),
                  const SizedBox(height: 16),
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: Text(
                      description,
                      style: context.textTheme.bodyLarge,
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),
            if (extraContent != null) ...[
              const Divider(height: 1),
              extraContent!,
            ],
            const Divider(height: 1),
            ConfirmButtons(
              primaryButton: ConfirmButton.accept(
                onPressed: onConfirmPressed,
                text: confirmButtonText,
                icon: confirmIcon,
              ),
              secondaryButton: ConfirmButton.reject(
                onPressed: onCancelPressed,
                text: cancelButtonText,
                icon: cancelIcon,
              ),
            ),
          ],
        ),
      ),
    );
  }

  static Future<bool> show(
    BuildContext context, {
    required String title,
    required String description,
    required String cancelButtonText,
    required String confirmButtonText,
    Color? confirmButtonColor,
    Widget? extraContent,
  }) async {
    final confirmed = await showModalBottomSheet<bool>(
      context: context,
      isScrollControlled: true,
      isDismissible: !context.isScreenReaderEnabled, // Avoid announcing the scrim
      builder: (BuildContext context) {
        return ConfirmActionSheet(
          title: title,
          description: description,
          cancelButtonText: cancelButtonText,
          confirmButtonText: confirmButtonText,
          onConfirmPressed: () => Navigator.pop(context, true),
          onCancelPressed: () => Navigator.pop(context, false),
          confirmButtonColor: confirmButtonColor,
          extraContent: extraContent,
        );
      },
    );
    return confirmed == true;
  }

  ElevatedButtonThemeData? buttonTheme(BuildContext context) {
    if (confirmButtonColor == null) return null;
    return ElevatedButtonThemeData(
      style: ElevatedButtonTheme.of(context).style?.copyWith(
            backgroundColor: MaterialStatePropertyAll(confirmButtonColor!),
          ),
    );
  }
}
