import 'package:flutter/material.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../widget/button/bottom_back_button.dart';
import '../widget/page_illustration.dart';
import '../widget/paragraphed_list.dart';
import '../widget/paragraphed_sliver_list.dart';
import '../widget/sliver_wallet_app_bar.dart';
import '../widget/wallet_scrollbar.dart';

class PlaceholderScreen extends StatelessWidget {
  final String? illustration;
  final String headline;

  /// Supports paragraphs through [ParagraphedList], i.e. the description is split using '\n\n'.
  final String description;

  const PlaceholderScreen({
    required this.headline,
    required this.description,
    this.illustration,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: WalletScrollbar(
          child: Column(
            children: [
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: headline,
                      scrollController: PrimaryScrollController.maybeOf(context),
                      automaticallyImplyLeading: true,
                    ),
                    SliverPadding(
                      sliver: ParagraphedSliverList.splitContent(description),
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                    ),
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: const EdgeInsets.symmetric(vertical: 24),
                        child: PageIllustration(
                          asset: illustration ?? WalletAssets.svg_placeholder,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
              const BottomBackButton(),
            ],
          ),
        ),
      ),
    );
  }

  static void show(
    BuildContext context, {
    bool secured = true,
    required String headline,
    required String description,
    String? illustration,
  }) {
    final errorScreen = PlaceholderScreen(
      headline: headline,
      description: description,
      illustration: illustration,
    );
    final route =
        secured ? SecuredPageRoute(builder: (c) => errorScreen) : MaterialPageRoute(builder: (c) => errorScreen);
    Navigator.push(context, route);
  }

  static void showGeneric(BuildContext context, {bool secured = true}) {
    show(
      context,
      secured: secured,
      headline: context.l10n.placeholderScreenHeadline,
      description: context.l10n.placeholderScreenGenericDescription,
      illustration: WalletAssets.svg_placeholder,
    );
  }

  static void showHelp(BuildContext context, {bool secured = true}) {
    show(
      context,
      secured: secured,
      headline: context.l10n.placeholderScreenHelpHeadline,
      description: context.l10n.placeholderScreenHelpDescription,
      illustration: WalletAssets.svg_placeholder,
    );
  }

  static void showContract(BuildContext context, {bool secured = true}) {
    show(
      context,
      secured: secured,
      headline: context.l10n.placeholderScreenHeadline,
      description: context.l10n.placeholderScreenContractDescription,
      illustration: WalletAssets.svg_signed,
    );
  }
}
