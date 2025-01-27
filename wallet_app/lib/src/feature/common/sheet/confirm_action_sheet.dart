import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/button/confirm/confirm_buttons.dart';
import '../widget/button/primary_button.dart';
import '../widget/button/secondary_button.dart';
import '../widget/text/body_text.dart';
import '../widget/text/title_text.dart';

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
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: TitleText(
                    title,
                    textAlign: TextAlign.start,
                  ),
                ),
                const SizedBox(height: 16),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: BodyText(
                    description,
                    textAlign: TextAlign.start,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 24),
            if (extraContent != null) ...[
              const Divider(),
              extraContent!,
            ],
            const Divider(),
            ConfirmButtons(
              primaryButton: PrimaryButton(
                key: const Key('acceptButton'),
                onPressed: onConfirmPressed,
                text: Text.rich(confirmButtonText.toTextSpan(context)),
                icon: confirmIcon == null ? null : Icon(confirmIcon),
              ),
              secondaryButton: SecondaryButton(
                key: const Key('rejectButton'),
                onPressed: onCancelPressed,
                text: Text.rich(cancelButtonText.toTextSpan(context)),
                icon: cancelIcon == null ? null : Icon(cancelIcon),
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
    return confirmed ?? false;
  }

  ElevatedButtonThemeData? buttonTheme(BuildContext context) {
    if (confirmButtonColor == null) return null;
    return ElevatedButtonThemeData(
      style: ElevatedButtonTheme.of(context).style?.copyWith(
            backgroundColor: WidgetStatePropertyAll(confirmButtonColor!),
          ),
    );
  }
}
