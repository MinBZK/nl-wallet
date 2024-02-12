import 'package:flutter/material.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../widget/button/bottom_back_button.dart';
import '../widget/wallet_app_bar.dart';

enum PlaceholderType { generic, contract }

class PlaceholderScreen extends StatelessWidget {
  final PlaceholderType type;

  const PlaceholderScreen({required this.type, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('placeholderScreen'),
      appBar: WalletAppBar(
        title: Text(context.l10n.placeholderScreenTitle),
      ),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return Column(
      children: [
        const Spacer(),
        Image.asset(
          _imageAssetName(),
          scale: context.isLandscape ? 1.5 : 1,
        ),
        const SizedBox(height: 16),
        Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            _informTitle(context),
            style: context.textTheme.displaySmall,
            textAlign: TextAlign.center,
          ),
        ),
        const Spacer(flex: 2),
        const BottomBackButton(),
      ],
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
