import 'package:flutter/material.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import '../widget/button/bottom_close_button.dart';
import '../widget/select_card_row.dart';
import '../widget/text/title_text.dart';
import '../widget/wallet_scrollbar.dart';

class SelectCardSheet extends StatelessWidget {
  final List<WalletCard> candidates;

  const SelectCardSheet({
    required this.candidates,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Padding(
                padding: const EdgeInsetsGeometry.symmetric(horizontal: 16, vertical: 24),
                child: TitleText(context.l10n.selectCardSheetTitle),
              ),
              const Divider(),
              _buildCardsSection(context),
            ],
          ),
          const BottomCloseButton(),
        ],
      ),
    );
  }

  Widget _buildCardsSection(BuildContext context) {
    return ListView.separated(
      itemBuilder: (context, index) => SelectCardRow(
        onPressed: () => Navigator.pop(context, candidates[index]),
        card: candidates[index],
      ),
      separatorBuilder: (context, index) => const Divider(),
      itemCount: candidates.length,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
    );
  }

  static Future<WalletCard?> show(BuildContext context, {required List<WalletCard> candidates}) async {
    return showModalBottomSheet<WalletCard>(
      context: context,
      isDismissible: !context.isScreenReaderEnabled, // Avoid announcing the scrim
      isScrollControlled: true,
      builder: (BuildContext context) {
        return WalletScrollbar(
          child: SingleChildScrollView(
            child: SelectCardSheet(candidates: candidates),
          ),
        );
      },
    );
  }
}
