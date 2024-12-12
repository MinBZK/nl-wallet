import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../widget/button/bottom_close_button.dart';
import '../widget/version/config_version_text.dart';
import '../widget/version/os_version_text.dart';
import '../widget/version/string_version_text.dart';
import '../widget/wallet_scrollbar.dart';

class ErrorDetailsSheet extends StatelessWidget {
  const ErrorDetailsSheet({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  context.l10n.errorDetailsSheetTitle,
                  style: context.textTheme.displayMedium,
                  textAlign: TextAlign.start,
                ),
                const SizedBox(height: 8),
                _buildInfoSection(context),
              ],
            ),
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
        StringVersionText(
          prefixTextStyle: context.textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold),
          alignHorizontal: false,
        ),
        const SizedBox(height: 4),
        OsVersionText(
          prefixTextStyle: context.textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold),
          alignHorizontal: false,
        ),
        const SizedBox(height: 4),
        ConfigVersionText(
          prefixTextStyle: context.textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold),
          alignHorizontal: false,
        ),
      ],
    );
  }

  static Future<void> show(BuildContext context) async {
    return showModalBottomSheet<void>(
      context: context,
      isDismissible: !context.isScreenReaderEnabled, // Avoid announcing the scrim
      isScrollControlled: true,
      builder: (BuildContext context) {
        return const WalletScrollbar(
          child: SingleChildScrollView(
            child: ErrorDetailsSheet(),
          ),
        );
      },
    );
  }
}
