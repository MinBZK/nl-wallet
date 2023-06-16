import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../navigation/wallet_routes.dart';
import '../common/widget/sliver_sized_box.dart';

const _kIllustration = 'assets/images/pin_timeout_illustration.png';

class PinBlockedScreen extends StatelessWidget {
  const PinBlockedScreen({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(locale.pinBlockedScreenTitle),
        leading: const SizedBox.shrink(),
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
                    _kIllustration,
                    width: double.infinity,
                    fit: BoxFit.fitWidth,
                  ),
                ),
                const SliverSizedBox(height: 24),
                SliverToBoxAdapter(
                  child: Text(
                    locale.pinBlockedScreenHeadline,
                    textAlign: TextAlign.start,
                    style: Theme.of(context).textTheme.displayMedium,
                  ),
                ),
                const SliverSizedBox(height: 8),
                SliverToBoxAdapter(
                  child: Text(locale.pinBlockedScreenDescription),
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
    final locale = AppLocalizations.of(context);
    return Container(
      alignment: Alignment.bottomCenter,
      padding: const EdgeInsets.symmetric(vertical: 24),
      child: ElevatedButton(
        onPressed: () => exit(0),
        child: Text(locale.pinBlockedScreenResetWalletCta),
      ),
    );
  }

  static void show(BuildContext context) {
    // Remove all routes and only keep the pinBlocked route
    Navigator.pushNamedAndRemoveUntil(context, WalletRoutes.pinBlockedRoute, (Route<dynamic> route) => false);
  }
}
