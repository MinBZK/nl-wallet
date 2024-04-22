import 'package:flutter/material.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../widget/button/bottom_back_button.dart';
import '../widget/sliver_sized_box.dart';
import '../widget/sliver_wallet_app_bar.dart';

enum PlaceholderType { generic, contract }

class PlaceholderScreen extends StatelessWidget {
  final PlaceholderType type;

  const PlaceholderScreen({required this.type, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('placeholderScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(child: _buildBody(context)),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        slivers: [
          SliverWalletAppBar(title: context.l10n.placeholderScreenTitle),
          const SliverSizedBox(height: 24),
          SliverToBoxAdapter(
            child: Image.asset(
              _imageAssetName(),
              height: 200,
              alignment: Alignment.center,
            ),
          ),
          const SliverSizedBox(height: 24),
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Text(
                _informTitle(context),
                style: context.textTheme.displaySmall,
                textAlign: TextAlign.center,
              ),
            ),
          ),
          const SliverSizedBox(height: 24),
        ],
      ),
    );
  }

  String _imageAssetName() {
    switch (type) {
      case PlaceholderType.generic:
        return WalletAssets.illustration_placeholder_generic;
      case PlaceholderType.contract:
        return WalletAssets.illustration_placeholder_contract;
    }
  }

  String _informTitle(BuildContext context) {
    switch (type) {
      case PlaceholderType.generic:
        return context.l10n.placeholderScreenGenericInformTitle;
      case PlaceholderType.contract:
        return context.l10n.placeholderScreenContractInformTitle;
    }
  }

  static void show(BuildContext context, {bool secured = true, PlaceholderType type = PlaceholderType.generic}) {
    Navigator.push(
      context,
      secured
          ? SecuredPageRoute(builder: (c) => PlaceholderScreen(type: type))
          : MaterialPageRoute(builder: (c) => PlaceholderScreen(type: type)),
    );
  }
}
