import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/bullet_list.dart';
import '../widget/button/tertiary_button.dart';
import '../widget/text/title_text.dart';
import '../widget/wallet_scrollbar.dart';

class MissingAttributesSheet extends StatelessWidget {
  final List<MissingAttribute> missingAttributes;

  const MissingAttributesSheet({
    required this.missingAttributes,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      minimum: const EdgeInsets.only(bottom: 24),
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
                TitleText(context.l10n.missingAttributesSheetTitle),
                const SizedBox(height: 16),
                TitleText(
                  context.l10n.missingAttributesSheetAttributesTitle,
                  style: context.textTheme.titleMedium,
                ),
                const SizedBox(height: 8),
                BulletList(
                  items: missingAttributes.map((it) => it.label.l10nValue(context)).toList(),
                  icon: _buildBulletListIcon(context),
                ),
              ],
            ),
          ),
          const Divider(),
          const SizedBox(height: 16),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: TertiaryButton(
              onPressed: () => Navigator.pop(context),
              text: Text.rich(context.l10n.generalClose.toTextSpan(context)),
              icon: const Icon(Icons.close),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildBulletListIcon(BuildContext context) {
    return Center(
      child: Container(
        height: 4,
        width: 4,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          color: context.theme.iconTheme.color,
        ),
      ),
    );
  }

  static Future<void> show(BuildContext context, List<MissingAttribute> missingAttributes) async {
    return showModalBottomSheet<void>(
      context: context,
      isDismissible: !context.isScreenReaderEnabled, // Avoid announcing the scrim
      isScrollControlled: true,
      builder: (BuildContext context) {
        return WalletScrollbar(
          child: SingleChildScrollView(
            child: MissingAttributesSheet(
              missingAttributes: missingAttributes,
            ),
          ),
        );
      },
    );
  }
}
