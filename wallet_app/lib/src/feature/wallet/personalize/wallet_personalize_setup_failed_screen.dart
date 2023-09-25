import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/wallet/reset_wallet_usecase.dart';
import '../../../navigation/secured_page_route.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/sliver_sized_box.dart';

class WalletPersonalizeSetupFailedScreen extends StatelessWidget {
  const WalletPersonalizeSetupFailedScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.walletPersonalizeSetupFailedScreenTitle),
      ),
      body: SafeArea(
        minimum: const EdgeInsets.only(bottom: 24),
        child: Scrollbar(
          thumbVisibility: true,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: CustomScrollView(
              slivers: [
                const SliverSizedBox(height: 24),
                SliverToBoxAdapter(
                  child: ExcludeSemantics(
                    child: Image.asset(
                      WalletAssets.illustration_digid_failure,
                      fit: context.isLandscape ? BoxFit.contain : BoxFit.fitWidth,
                      height: context.isLandscape ? 160 : null,
                      width: double.infinity,
                    ),
                  ),
                ),
                const SliverSizedBox(height: 24),
                SliverToBoxAdapter(
                  child: MergeSemantics(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          context.l10n.walletPersonalizeSetupFailedScreenHeadline,
                          textAlign: TextAlign.start,
                          style: context.textTheme.displaySmall,
                        ),
                        const SizedBox(height: 8),
                        Text(
                          context.l10n.walletPersonalizeSetupFailedScreenDescription,
                          textAlign: TextAlign.start,
                          style: context.textTheme.bodyLarge,
                        ),
                      ],
                    ),
                  ),
                ),
                const SliverSizedBox(height: 32),
                SliverFillRemaining(
                  hasScrollBody: false,
                  fillOverscroll: true,
                  child: _buildBottomSection(context),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        const SizedBox(height: 24),
        ElevatedButton(
          onPressed: () async {
            final navigator = Navigator.of(context);
            await context.read<ResetWalletUseCase>().invoke();
            navigator.restorablePushNamedAndRemoveUntil(
              WalletRoutes.setupSecurityRoute,
              ModalRoute.withName(WalletRoutes.splashRoute),
            );
          },
          child: Text(context.l10n.walletPersonalizeSetupFailedScreenCta),
        ),
      ],
    );
  }

  static void show(BuildContext context) {
    Navigator.pushAndRemoveUntil(
      context,
      SecuredPageRoute(builder: (c) => const WalletPersonalizeSetupFailedScreen()),
      ModalRoute.withName(WalletRoutes.splashRoute),
    );
  }
}
