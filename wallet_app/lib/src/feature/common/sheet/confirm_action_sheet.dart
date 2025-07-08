import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/button/confirm/confirm_buttons.dart';
import '../widget/button/primary_button.dart';
import '../widget/button/secondary_button.dart';
import '../widget/text/body_text.dart';
import '../widget/text/title_text.dart';

/// A widget that displays a modal bottom sheet for confirming an action.
///
/// It typically includes a title, a descriptive message, and two buttons:
/// one for confirming the action and one for canceling it.
///
/// This widget is usually displayed via the static [show] method, which
/// handles the presentation of the modal bottom sheet.
class ConfirmActionSheet extends StatelessWidget {
  /// The text displayed as the main heading of the action sheet.
  final String title;

  /// The detailed message displayed below the title, explaining the action.
  final String description;

  /// Configuration for the primary (confirm) button.
  final ConfirmSheetButtonStyle confirmButton;

  /// Callback executed when the confirm button is pressed.
  ///
  /// Typically, this will pop the navigator with a `true` value.
  final VoidCallback? onConfirmPressed;

  /// Configuration for the secondary (cancel) button.
  final ConfirmSheetButtonStyle cancelButton;

  /// Callback executed when the cancel button is pressed.
  ///
  /// Typically, this will pop the navigator with a `false` value.
  final VoidCallback? onCancelPressed;

  /// An optional widget to display between the description and the buttons.
  final Widget? extraContent;

  const ConfirmActionSheet({
    required this.title,
    required this.description,
    required this.confirmButton,
    this.onConfirmPressed,
    required this.cancelButton,
    this.onCancelPressed,
    this.extraContent,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Theme(
      data: context.theme.copyWith(
        elevatedButtonTheme: _generatePrimaryButtonTheme(context),
        outlinedButtonTheme: _generateSecondaryButtonTheme(context),
      ), // Custom theme override to provide (optional) background color
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
                text: Text.rich(confirmButton.cta.toTextSpan(context)),
                icon: confirmButton.icon == null ? null : Icon(confirmButton.icon),
              ),
              secondaryButton: SecondaryButton(
                key: const Key('rejectButton'),
                onPressed: onCancelPressed,
                text: Text.rich(cancelButton.cta.toTextSpan(context)),
                icon: cancelButton.icon == null ? null : Icon(cancelButton.icon),
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
  /// The [confirmButton] is a [ConfirmSheetButtonStyle] to configure the confirm button.
  /// The [cancelButton] is a [ConfirmSheetButtonStyle] to configure the cancel button.
  /// The [extraContent] (optional) is a widget to display between the description and the buttons,
  /// typically used for additional options or information.
  static Future<bool> show(
    BuildContext context, {
    required String title,
    required String description,
    required ConfirmSheetButtonStyle confirmButton,
    required ConfirmSheetButtonStyle cancelButton,
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
          confirmButton: confirmButton,
          onConfirmPressed: () => Navigator.pop(context, true),
          cancelButton: cancelButton,
          onCancelPressed: () => Navigator.pop(context, false),
          extraContent: extraContent,
        );
      },
    );
    return confirmed ?? false;
  }

  ElevatedButtonThemeData? _generatePrimaryButtonTheme(BuildContext context) {
    if (confirmButton.color == null) return null;
    return ElevatedButtonThemeData(
      style: ElevatedButtonTheme.of(context).style?.copyWith(
            backgroundColor: WidgetStatePropertyAll(confirmButton.color!),
          ),
    );
  }

  OutlinedButtonThemeData? _generateSecondaryButtonTheme(BuildContext context) {
    if (cancelButton.color == null) return null;
    return OutlinedButtonThemeData(
      style: OutlinedButtonTheme.of(context).style?.copyWith(
            backgroundColor: WidgetStatePropertyAll(cancelButton.color!),
          ),
    );
  }
}

/// A configuration object for a button in a [ConfirmActionSheet].
class ConfirmSheetButtonStyle {
  /// The text displayed on the button.
  final String cta;

  /// The icon displayed next to the text on the button (optional).
  final IconData? icon;

  /// The background color of the button (optional).
  /// If not provided, the default theme color will be used.
  final Color? color;

  const ConfirmSheetButtonStyle({
    required this.cta,
    this.icon,
    this.color,
  });
}
