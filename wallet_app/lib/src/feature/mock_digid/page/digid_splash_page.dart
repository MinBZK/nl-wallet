import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

/// Force highest res version here, avoids bloating the assets with files that are temporary by nature.
const _kDigidLogoPath = 'assets/images/3.0x/digid_logo.png';

class DigidSplashPage extends StatelessWidget {
  const DigidSplashPage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Stack(
        children: [
          Align(
            alignment: Alignment.topCenter,
            child: Image.asset('assets/non-free/images/logo_rijksoverheid_label.png'),
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
                Image.asset(_kDigidLogoPath),
                const SizedBox(width: 32),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        context.l10n.mockDigidScreenTitle,
                        style: context.textTheme.displayMedium?.copyWith(color: Colors.black),
                      ),
                    ],
                  ),
                )
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
