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
                  child: TitleText(title),
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

  /// Shows a modal bottom sheet with a title, description, and two buttons
  /// (confirm and cancel).
  ///
  /// Returns a [Future] that resolves to `true` if the confirm button is pressed,
  /// and `false` if the cancel button is pressed or the sheet is dismissed.
  ///
  /// The [context] is the `BuildContext` from which to show the sheet.
  /// The [title] is the text displayed at the top of the sheet.
  /// The [description] is the text displayed below the title.
  /// The [confirmButtonText] is the text for the confirm button.
  /// The [confirmButtonColor] (optional) is the background color for the confirm button.
  /// The [confirmIcon] (optional) is an icon to display next to the confirm button text.
  /// The [cancelButtonText] is the text for the cancel button.
  /// The [cancelIcon] (optional) is an icon to display next to the cancel button text.
  /// The [extraContent] (optional) is a widget to display between the description and the buttons,
  /// typically used for additional options or information.
  static Future<bool> show(
    BuildContext context, {
    required String title,
    required String description,
    required String confirmButtonText,
    Color? confirmButtonColor,
    IconData? confirmIcon,
    required String cancelButtonText,
    IconData? cancelIcon,
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
          confirmButtonText: confirmButtonText,
          confirmButtonColor: confirmButtonColor,
          confirmIcon: confirmIcon,
          onConfirmPressed: () => Navigator.pop(context, true),
          cancelButtonText: cancelButtonText,
          cancelIcon: cancelIcon,
          onCancelPressed: () => Navigator.pop(context, false),
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
