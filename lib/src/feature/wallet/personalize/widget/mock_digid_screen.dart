import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

/// Force highest res version here, avoids bloating the assets with files that are temporary by nature.
const _kDigidLogoPath = 'assets/images/3.0x/digid_logo.png';

/// Screen that can be navigated to when DigiD authentication is to be faked.
/// Most likely used via 'await MockDigidScreen.show(context);`
class MockDigidScreen extends StatelessWidget {
  const MockDigidScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Stack(
        children: [
          Align(
            alignment: Alignment.topCenter,
            child: Container(
              height: 100,
              width: 50,
              color: const Color.fromARGB(255, 48, 81, 130),
            ),
          ),
          SafeArea(child: _buildBody(context)),
        ],
      ),
      backgroundColor: const Color(0xFFD2762B),
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
                        style: Theme.of(context).textTheme.headline3?.copyWith(color: Colors.black),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        locale.mockDigidScreenDescription,
                        style: Theme.of(context).textTheme.bodyText1?.copyWith(color: Colors.black),
                      ),
                    ],
                  ),
                )
              ],
            ),
          ),
          const Spacer(),
          ElevatedButton(
            onPressed: () => Navigator.pop(context),
            child: Text(locale.mockDigidScreenCta),
          ),
          const SizedBox(height: 8),
        ],
      ),
    );
  }

  static Future<void> show(BuildContext context) {
    return Navigator.of(context).push(MaterialPageRoute(builder: (context) {
      return const MockDigidScreen();
    }));
  }
}
