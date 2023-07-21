import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/icon_row.dart';

const _kCoverHeaderLabelImage = 'assets/non-free/images/logo_rijksoverheid_label.png';

class IntroductionPrivacyPage extends StatelessWidget {
  final Widget? footer;

  const IntroductionPrivacyPage({
    this.footer,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        children: [
          Expanded(
            child: Scrollbar(
              thumbVisibility: true,
              child: ListView(
                padding: EdgeInsets.zero,
                children: [
                  Align(
                    alignment: Alignment.topCenter,
                    child: Image.asset(_kCoverHeaderLabelImage, fit: BoxFit.cover),
                  ),
                  const SizedBox(height: 48),
                  _buildInfoSection(context),
                ],
              ),
            ),
          ),
          if (footer != null) footer!,
        ],
      ),
    );
  }

  Widget _buildInfoSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            context.l10n.introductionPrivacyPageTitle,
            style: context.textTheme.displayLarge,
            textAlign: TextAlign.start,
            textScaleFactor: 1,
          ),
          const SizedBox(height: 20),
          IconRow(
            padding: const EdgeInsets.symmetric(vertical: 4),
            icon: Icon(
              Icons.check,
              color: context.colorScheme.primary,
            ),
            text: Text(context.l10n.introductionPrivacyPageBullet1),
          ),
          IconRow(
            padding: const EdgeInsets.symmetric(vertical: 4),
            icon: Icon(
              Icons.check,
              color: context.colorScheme.primary,
            ),
            text: Text(context.l10n.introductionPrivacyPageBullet2),
          ),
          IconRow(
            padding: const EdgeInsets.symmetric(vertical: 4),
            icon: Icon(
              Icons.check,
              color: context.colorScheme.primary,
            ),
            text: Text(context.l10n.introductionPrivacyPageBullet3),
          ),
        ],
      ),
    );
  }
}
