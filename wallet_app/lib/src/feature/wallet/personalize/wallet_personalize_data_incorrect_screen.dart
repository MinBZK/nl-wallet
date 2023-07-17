import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

import '../../../../environment.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/info_row.dart';
import '../../common/screen/placeholder_screen.dart';

class WalletPersonalizeDataIncorrectScreen extends StatelessWidget {
  const WalletPersonalizeDataIncorrectScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.walletPersonalizeDataIncorrectScreenTitle),
      ),
      body: CustomScrollView(
        slivers: [
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
              child: MergeSemantics(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      context.l10n.walletPersonalizeDataIncorrectScreenSubhead,
                      style: context.textTheme.displayMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      context.l10n.walletPersonalizeDataIncorrectScreenDescription,
                      style: context.textTheme.bodyLarge,
                    ),
                  ],
                ),
              ),
            ),
          ),
          SliverToBoxAdapter(child: _buildOptionalMunicipalitySection(context)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: _buildBottomSection(context),
          ),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return const Align(
      alignment: Alignment.bottomCenter,
      child: BottomBackButton(),
    );
  }

  static void show(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (c) => const WalletPersonalizeDataIncorrectScreen()),
    );
  }

  Widget _buildOptionalMunicipalitySection(BuildContext context) {
    if (!Environment.mockRepositories) return const SizedBox.shrink();
    // FIXME: Den Haag (and corresponding data) is hardcoded here for the
    // FIXME: mock build. Actual implementation should rely on provided
    // FIXME: municipality, but this feature is to be refined.
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Divider(height: 64),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            'Gemeente Den Haag',
            style: context.textTheme.titleMedium,
          ),
        ),
        const SizedBox(height: 16),
        InfoRow(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          leading: Icon(
            Icons.language_outlined,
            color: context.colorScheme.onSurface,
          ),
          title: Text(
            context.l10n.walletPersonalizeDataIncorrectScreenWebsite,
            style: context.textTheme.bodySmall,
          ),
          subtitle: Text.rich(
            TextSpan(
              children: [
                TextSpan(
                  text: 'www.denhaag.nl',
                  recognizer: TapGestureRecognizer()..onTap = () => PlaceholderScreen.show(context, secured: false),
                  style: context.textTheme.bodyLarge?.copyWith(
                    decoration: TextDecoration.underline,
                    color: context.colorScheme.primary,
                  ),
                )
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        InfoRow(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          leading: Icon(
            Icons.phone_outlined,
            color: context.colorScheme.onSurface,
          ),
          title: Text(
            context.l10n.walletPersonalizeDataIncorrectScreenPhone,
            style: context.textTheme.bodySmall,
          ),
          subtitle: Text(
            '+31 70 353 30 00',
            style: context.textTheme.bodyLarge,
          ),
        ),
        const Divider(height: 64),
      ],
    );
  }
}
