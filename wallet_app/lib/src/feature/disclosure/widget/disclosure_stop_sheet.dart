import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/sheet/confirm_action_sheet.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/wallet_scrollbar.dart';

/// Builds upon the [ConfirmActionSheet], but supplies defaults for
/// when the user is requesting to stop the disclosure flow.
class DisclosureStopSheet extends StatelessWidget {
  /// Callback invoked when the confirm action (stop) is pressed.
  final VoidCallback onConfirmPressed;

  /// Callback invoked when the cancel action is pressed.
  final VoidCallback onCancelPressed;

  /// Optional callback invoked when the report issue action is pressed.
  /// If null, the report issue button will not be shown.
  final VoidCallback? onReportIssuePressed;

  /// The name of the organization involved in the disclosure, if applicable.
  /// This is used in the description text.
  final String? organizationName;

  /// The type of description to display in the sheet.
  /// This influences the text shown to the user.
  final StopDescriptionType descriptionType;

  const DisclosureStopSheet({
    required this.organizationName,
    this.onReportIssuePressed,
    required this.onCancelPressed,
    required this.onConfirmPressed,
    this.descriptionType = StopDescriptionType.generic,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ConfirmActionSheet(
      title: context.l10n.disclosureStopSheetTitle,
      description: _resolveDescription(context),
      confirmButton: ConfirmSheetButtonStyle(
        cta: context.l10n.disclosureStopSheetPositiveCta,
        color: context.colorScheme.error,
        icon: Icons.not_interested,
      ),
      cancelButton: ConfirmSheetButtonStyle(
        cta: context.l10n.disclosureStopSheetNegativeCta,
        icon: Icons.arrow_back,
      ),
      onCancelPressed: onCancelPressed,
      onConfirmPressed: onConfirmPressed,
      extraContent: onReportIssuePressed == null
          ? null
          : ListButton(
              dividerSide: DividerSide.none,
              onPressed: onReportIssuePressed,
              text: Text.rich(context.l10n.disclosureStopSheetReportIssueCta.toTextSpan(context)),
            ),
    );
  }

  String _resolveDescription(BuildContext context) {
    switch (descriptionType) {
      case StopDescriptionType.forUrlCheck:
        return context.l10n.disclosureStopSheetDescriptionForUrlCheck;
      case StopDescriptionType.forLogin:
        return organizationName == null
            ? context.l10n.disclosureStopSheetDescriptionForLoginVariant
            : context.l10n.disclosureStopSheetDescriptionForLogin(organizationName!);
      case StopDescriptionType.generic:
        return organizationName == null
            ? context.l10n.disclosureStopSheetDescriptionVariant
            : context.l10n.disclosureStopSheetDescription(organizationName!);
    }
  }

  static Future<bool> show(
    BuildContext context, {
    LocalizedText? organizationName,
    VoidCallback? onReportIssuePressed,
    StopDescriptionType descriptionType = StopDescriptionType.generic,
  }) async {
    final confirmed = await showModalBottomSheet<bool>(
      context: context,
      isDismissible: !context.isScreenReaderEnabled, // Avoid announcing the scrim
      isScrollControlled: true,
      builder: (BuildContext context) {
        return WalletScrollbar(
          child: SingleChildScrollView(
            child: DisclosureStopSheet(
              organizationName: organizationName?.l10nValue(context),
              onReportIssuePressed: onReportIssuePressed,
              onConfirmPressed: () => Navigator.pop(context, true),
              onCancelPressed: () => Navigator.pop(context, false),
              descriptionType: descriptionType,
            ),
          ),
        );
      },
    );
    return confirmed ?? false;
  }
}

enum StopDescriptionType { forUrlCheck, forLogin, generic }
