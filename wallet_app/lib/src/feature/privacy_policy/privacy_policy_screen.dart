import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_markdown/flutter_markdown.dart';

import '../../theme/base_wallet_theme.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/object_extension.dart';
import '../../util/launch_util.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

class PrivacyPolicyScreen extends StatelessWidget {
  const PrivacyPolicyScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: const WalletAppBar(
        // hardcoded dutch title to match hardcoded dutch policy
        title: TitleText('Privacyverklaring Publieke Wallet (eerste beproevingsperiode)'),
        leading: BackIconButton(),
      ),
      key: const Key('privacyPolicyScreen'),
      body: SafeArea(child: _buildContent(context)),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Column(
      children: [
        Expanded(
          child: WalletScrollbar(
            child: CustomScrollView(
              slivers: [
                const SliverToBoxAdapter(
                  child: Padding(
                    padding: kDefaultTitlePadding,
                    child: TitleText('Privacyverklaring Publieke Wallet (eerste beproevingsperiode)'),
                  ),
                ),
                SliverPadding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  sliver: SliverToBoxAdapter(
                    child: FutureBuilder<String>(
                      future: rootBundle.loadString('assets/non-free/markdown/policy.md'),
                      builder: (context, snapshot) {
                        if (!snapshot.hasData) {
                          return const Padding(
                            padding: EdgeInsets.all(42),
                            child: CenteredLoadingIndicator(),
                          );
                        }
                        return MarkdownBody(
                          data: snapshot.data!,
                          onTapLink: (string, href, title) => href?.let(launchUrlStringCatching),
                          styleSheet: MarkdownStyleSheet(
                            textScaler: context.textScaler,
                            p: context.textTheme.bodyLarge,
                            h1: context.textTheme.displayMedium,
                            h2: context.textTheme.labelLarge,
                            h3: context.textTheme.bodyLarge?.copyWith(
                              fontVariations: [BaseWalletTheme.fontVariationBold],
                            ),
                            h6: context.textTheme.labelMedium,
                            a: context.textTheme.bodyLarge?.copyWith(
                              fontVariations: [BaseWalletTheme.fontVariationRegular],
                              decoration: TextDecoration.underline,
                              color: context.colorScheme.primary,
                            ),
                          ),
                        );
                      },
                    ),
                  ),
                ),
                const SliverSizedBox(height: 24),
              ],
            ),
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }
}
