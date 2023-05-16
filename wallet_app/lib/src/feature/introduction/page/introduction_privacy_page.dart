import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

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
    return PrimaryScrollController(
      controller: ScrollController(),
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
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            locale.introductionPrivacyPageTitle,
            style: Theme.of(context).textTheme.displayLarge,
            textAlign: TextAlign.start,
            textScaleFactor: 1,
          ),
          const SizedBox(height: 20),
          IconRow(
            padding: const EdgeInsets.symmetric(vertical: 4),
            icon: Icon(
              Icons.check,
              color: Theme.of(context).colorScheme.primary,
            ),
            text: Text(locale.introductionPrivacyPageBullet1),
          ),
          IconRow(
            padding: const EdgeInsets.symmetric(vertical: 4),
            icon: Icon(
              Icons.check,
              color: Theme.of(context).colorScheme.primary,
            ),
            text: Text(locale.introductionPrivacyPageBullet2),
          ),
          IconRow(
            padding: const EdgeInsets.symmetric(vertical: 4),
            icon: Icon(
              Icons.check,
              color: Theme.of(context).colorScheme.primary,
            ),
            text: Text(locale.introductionPrivacyPageBullet3),
          ),
        ],
      ),
    );
  }
}
