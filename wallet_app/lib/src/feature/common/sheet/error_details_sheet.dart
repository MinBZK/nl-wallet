import 'package:flutter/material.dart';

import '../../../domain/model/result/application_error.dart';
import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../widget/button/bottom_close_button.dart';
import '../widget/text/title_text.dart';
import '../widget/version/application_error_text.dart';
import '../widget/version/config_version_text.dart';
import '../widget/version/os_version_text.dart';
import '../widget/version/string_version_text.dart';
import '../widget/wallet_scrollbar.dart';

class ErrorDetailsSheet extends StatelessWidget {
  final ApplicationError? error;

  const ErrorDetailsSheet({
    this.error,
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
                TitleText(context.l10n.errorDetailsSheetTitle),
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
    final TextStyle? prefixStyle =
        context.textTheme.bodyMedium?.copyWith(fontVariations: [BaseWalletTheme.fontVariationBold]);
    final items = <Widget>[
      StringVersionText(
        prefixTextStyle: prefixStyle,
        alignHorizontal: false,
      ),
      OsVersionText(
        prefixTextStyle: prefixStyle,
        alignHorizontal: false,
      ),
      ConfigVersionText(
        prefixTextStyle: prefixStyle,
        alignHorizontal: false,
      ),
      if (error != null)
        ApplicationErrorText(
          error: error!,
          prefixTextStyle: prefixStyle,
          alignHorizontal: false,
        ),
    ];

    return ListView.separated(
      itemBuilder: (context, index) => items[index],
      separatorBuilder: (context, index) => const SizedBox(height: 4),
      itemCount: items.length,
      shrinkWrap: true,
    );
  }

  static Future<void> show(BuildContext context, {ApplicationError? error}) async {
    return showModalBottomSheet<void>(
      context: context,
      isDismissible: !context.isScreenReaderEnabled, // Avoid announcing the scrim
      isScrollControlled: true,
      builder: (BuildContext context) {
        return WalletScrollbar(
          child: SingleChildScrollView(
            child: ErrorDetailsSheet(error: error),
          ),
        );
      },
    );
  }
}
