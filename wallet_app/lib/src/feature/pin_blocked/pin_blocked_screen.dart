import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/wallet/reset_wallet_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/wallet_app_bar.dart';

class PinBlockedScreen extends StatelessWidget {
  const PinBlockedScreen({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: Text(context.l10n.pinBlockedScreenTitle),
        automaticallyImplyLeading: false,
      ),
      body: PrimaryScrollController(
        controller: ScrollController(),
        child: Scrollbar(
          thumbVisibility: true,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: CustomScrollView(
              slivers: [
                const SliverSizedBox(height: 24),
                SliverToBoxAdapter(
                  child: Image.asset(
                    WalletAssets.illustration_pin_timeout,
                    width: double.infinity,
                    fit: BoxFit.fitWidth,
                  ),
                ),
                const SliverSizedBox(height: 24),
                SliverToBoxAdapter(
                  child: Text(
                    context.l10n.pinBlockedScreenHeadline,
                    textAlign: TextAlign.start,
                    style: context.textTheme.displayMedium,
                  ),
                ),
                const SliverSizedBox(height: 8),
                SliverToBoxAdapter(
                  child: Text(context.l10n.pinBlockedScreenDescription),
                ),
                SliverFillRemaining(
                  hasScrollBody: false,
                  fillOverscroll: true,
                  child: _buildBottomSection(context),
                )
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Container(
      alignment: Alignment.bottomCenter,
      padding: const EdgeInsets.symmetric(vertical: 24),
      child: ElevatedButton(
        onPressed: () async {
          final navigator = Navigator.of(context);
          await context.read<ResetWalletUseCase>().invoke();
          navigator.restorablePushNamedAndRemoveUntil(
            WalletRoutes.setupSecurityRoute,
            ModalRoute.withName(WalletRoutes.splashRoute),
          );
        },
        child: Text(context.l10n.pinBlockedScreenResetWalletCta),
      ),
    );
  }

  static void show(BuildContext context) {
    // Remove all routes and only keep the pinBlocked route
    Navigator.pushNamedAndRemoveUntil(context, WalletRoutes.pinBlockedRoute, (Route<dynamic> route) => false);
  }
}
