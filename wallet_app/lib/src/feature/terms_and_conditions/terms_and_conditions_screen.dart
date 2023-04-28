import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/policy/policy.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/placeholder_screen.dart';
import '../common/widget/policy/extended_policy_row.dart';

class TermsAndConditionsScreen extends StatelessWidget {
  final Policy policy;

  const TermsAndConditionsScreen({required this.policy, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(locale.termsAndConditionsScreenTitle),
      ),
      body: Column(
        children: [
          Expanded(child: _buildContent(context)),
          const Divider(height: 1),
          const BottomBackButton(),
        ],
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scrollbar(
      thumbVisibility: true,
      child: ListView(
        children: [
          ExtendedPolicyRow(icon: Icons.flag_outlined, title: locale.termsAndConditionsScreenPurposeTitle),
          const Divider(height: 1),
          ExtendedPolicyRow(icon: Icons.share_outlined, title: locale.termsAndConditionsScreenShareTitle),
          const Divider(height: 1),
          ExtendedPolicyRow(icon: Icons.access_time_outlined, title: locale.termsAndConditionsScreenStorageTitle),
          const Divider(height: 1),
          ExtendedPolicyRow(icon: Icons.delete_outline, title: locale.termsAndConditionsScreenDeleteTitle),
          const Divider(height: 1),
          _buildDiscoverMoreSection(context),
          const Divider(height: 1),
          const SizedBox(height: 32),
        ],
      ),
    );
  }

  Widget _buildDiscoverMoreSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final subtitleTheme = Theme.of(context).textTheme.bodyLarge;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            locale.termsAndConditionsScreenDiscoverMoreTitle,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          Text.rich(
            TextSpan(
              style: subtitleTheme,
              text: locale.termsAndConditionsScreenDiscoverMoreSubtitlePart1,
              children: [
                TextSpan(
                  text: locale.termsAndConditionsScreenDiscoverMoreSubtitlePart2,
                  style: subtitleTheme?.copyWith(
                    color: Theme.of(context).colorScheme.primary,
                    decoration: TextDecoration.underline,
                  ),
                  recognizer: TapGestureRecognizer()..onTap = () => PlaceholderScreen.show(context),
                ),
                TextSpan(text: locale.termsAndConditionsScreenDiscoverMoreSubtitlePart3),
              ],
            ),
          ),
        ],
      ),
    );
  }

  static void show(BuildContext context, Policy policy) {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (c) => TermsAndConditionsScreen(policy: policy)),
    );
  }
}
