import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

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
      backgroundColor: Theme.of(context).primaryColor,
    );
  }

  Widget _buildBody(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const Spacer(),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 32.0),
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
                        locale.mockDigidScreenTitle,
                        style: Theme.of(context).textTheme.headline2?.copyWith(color: Colors.black),
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
