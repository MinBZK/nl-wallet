import 'dart:io';
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/page_illustration.dart';
import '../../common/widget/paragraphed_list.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_scrollbar.dart';

class WalletTransferSourceTransferSuccessPage extends StatelessWidget {
  const WalletTransferSourceTransferSuccessPage({super.key});

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      bottom: false,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildScrollableSection(context),
          _buildBottomSection(context),
        ],
      ),
    );
  }

  Widget _buildScrollableSection(BuildContext context) {
    return Expanded(
      child: WalletScrollbar(
        child: ListView(
          padding: const EdgeInsets.symmetric(vertical: 12),
          children: [
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TitleText(context.l10n.walletTransferSourceScreenSuccessTitle),
                  const SizedBox(height: 8),
                  ParagraphedList.splitContent(
                    context.l10n.walletTransferSourceScreenSuccessDescription,
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),
            const PageIllustration(asset: WalletAssets.svg_move_source_success),
          ],
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    if (!Platform.isAndroid) {
      // iOS does not support gracefully closing the app
      return const SizedBox.shrink();
    }
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Divider(),
        const SizedBox(height: 24),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: PrimaryButton(
            text: Text(context.l10n.generalClose),
            icon: const Icon(Icons.close_outlined),
            onPressed: SystemNavigator.pop,
          ),
        ),
        SizedBox(height: max(24, context.mediaQuery.viewPadding.bottom)),
      ],
    );
  }
}
