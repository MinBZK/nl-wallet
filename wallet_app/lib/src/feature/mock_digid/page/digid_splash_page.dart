// coverage:ignore-file
import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';

class DigidSplashPage extends StatelessWidget {
  const DigidSplashPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Stack(
        children: [
          Align(
            alignment: Alignment.topCenter,
            child: Image.asset(WalletAssets.logo_rijksoverheid_label),
          ),
          SafeArea(child: _buildBody(context)),
        ],
      ),
      backgroundColor: context.colorScheme.primary,
    );
  }

  Widget _buildBody(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const Spacer(),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 32),
            child: Row(
              children: [
                Image.asset(WalletAssets.logo_digid_large),
                const SizedBox(width: 32),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        context.l10n.mockDigidScreenTitle,
                        style: context.textTheme.headlineLarge?.copyWith(color: Colors.black),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          const Spacer(),
          const SizedBox(height: 8),
        ],
      ),
    );
  }
}
