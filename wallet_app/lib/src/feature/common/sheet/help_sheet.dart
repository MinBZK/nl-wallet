import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../screen/placeholder_screen.dart';
import '../widget/button/bottom_close_button.dart';
import '../widget/button/list_button.dart';
import '../widget/version/app_version_text.dart';
import '../widget/version/config_version_text.dart';
import '../widget/version/os_version_text.dart';
import '../widget/wallet_scrollbar.dart';

class HelpSheet extends StatelessWidget {
  final String? errorCode, supportCode;

  const HelpSheet({
    this.errorCode,
    this.supportCode,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          MergeSemantics(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    context.l10n.helpSheetTitle,
                    style: context.textTheme.displayMedium,
                    textAlign: TextAlign.start,
                  ),
                  const SizedBox(height: 16),
                  Text(
                    context.l10n.helpSheetDescription,
                    style: context.textTheme.bodyLarge,
                  ),
                  const SizedBox(height: 16),
                  _buildInfoSection(context),
                ],
              ),
            ),
          ),
          ListButton(
            dividerSide: DividerSide.top,
            text: Text.rich(context.l10n.helpSheetHelpdeskCta.toTextSpan(context)),
            onPressed: () => PlaceholderScreen.showGeneric(context, secured: false),
          ),
          const BottomCloseButton(),
        ],
      ),
    );
  }

  Widget _buildInfoSection(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        AppVersionText(
          prefixTextStyle: context.textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 4),
        OsVersionText(
          prefixTextStyle: context.textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 4),
        ConfigVersionText(
          prefixTextStyle: context.textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 4),
        errorCode == null
            ? const SizedBox.shrink()
            : Text(
                context.l10n.helpSheetErrorCode(errorCode!),
                style: context.textTheme.bodyMedium
                    ?.copyWith(fontWeight: FontWeight.bold, color: context.colorScheme.error),
              ),
        if (supportCode != null) const SizedBox(height: 4),
        supportCode == null
            ? const SizedBox.shrink()
            : Text(
                context.l10n.helpSheetSupportCode(supportCode!),
                style: context.textTheme.bodyMedium
                    ?.copyWith(fontWeight: FontWeight.bold, color: context.colorScheme.error),
              ),
      ],
    );
  }

  static Future<void> show(
    BuildContext context, {
    String? errorCode,
    String? supportCode,
  }) async {
    return showModalBottomSheet<void>(
      context: context,
      isDismissible: !context.isScreenReaderEnabled, // Avoid announcing the scrim
      isScrollControlled: true,
      builder: (BuildContext context) {
        return WalletScrollbar(
          child: SingleChildScrollView(
            child: HelpSheet(
              errorCode: errorCode,
              supportCode: supportCode,
            ),
          ),
        );
      },
    );
  }
}
