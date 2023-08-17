import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/button/rounded_back_button.dart';
import '../common/widget/sliver_sized_box.dart';

class IntroductionExpectationsScreen extends StatelessWidget {
  const IntroductionExpectationsScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: context.colorScheme.inverseSurface,
      body: SafeArea(
        child: Column(
          children: [
            _buildBackButton(context),
            Expanded(child: _buildContent(context)),
            _buildCreateWalletButton(context),
          ],
        ),
      ),
    );
  }

  Widget _buildBackButton(BuildContext context) {
    return const Align(
      alignment: Alignment.centerLeft,
      child: RoundedBackButton(),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        slivers: <Widget>[
          const SliverSizedBox(height: 20),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            sliver: SliverToBoxAdapter(
              child: Text(
                context.l10n.introductionExpectationsScreenTitle,
                style: context.textTheme.displayLarge,
              ),
            ),
          ),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            sliver: SliverToBoxAdapter(
              child: _buildExpectationStep(
                context,
                1,
                context.l10n.introductionExpectationsScreenStep1,
              ),
            ),
          ),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            sliver: SliverToBoxAdapter(
              child: _buildExpectationStep(
                context,
                2,
                context.l10n.introductionExpectationsScreenStep2,
              ),
            ),
          ),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            sliver: SliverToBoxAdapter(
              child: _buildExpectationStep(
                context,
                3,
                context.l10n.introductionExpectationsScreenStep3,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildExpectationStep(BuildContext context, int step, String description) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: context.colorScheme.background,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        children: [
          Text(
            '$step.',
            style: context.textTheme.displaySmall,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              description,
              style: context.textTheme.bodyLarge,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCreateWalletButton(BuildContext context) {
    return Padding(
      padding: EdgeInsets.symmetric(horizontal: 16, vertical: context.isLandscape ? 8 : 24),
      child: ElevatedButton(
        onPressed: () => Navigator.of(context).restorablePushNamed(WalletRoutes.introductionPrivacyRoute),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.arrow_forward, size: 16),
            const SizedBox(width: 8),
            Text(context.l10n.introductionExpectationsScreenCta),
          ],
        ),
      ),
    );
  }
}
