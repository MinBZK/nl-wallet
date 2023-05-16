import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/placeholder_screen.dart';

const _kPinHeaderImage = 'assets/images/forgot_pin_header.png';

class ForgotPinScreen extends StatelessWidget {
  const ForgotPinScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(locale.forgotPinScreenTitle),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(child: _buildScrollableSection(context)),
            _buildBottomSection(context),
          ],
        ),
      ),
    );
  }

  Widget _buildScrollableSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scrollbar(
      thumbVisibility: true,
      child: ListView(
        padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
        children: [
          Image.asset(_kPinHeaderImage, fit: BoxFit.fitWidth),
          const SizedBox(height: 24),
          Text(
            locale.forgotPinScreenHeadline,
            textAlign: TextAlign.start,
            style: Theme.of(context).textTheme.displayMedium,
          ),
          const SizedBox(height: 8),
          Text(
            locale.forgotPinScreenDescription,
            textAlign: TextAlign.start,
            style: Theme.of(context).textTheme.bodyLarge,
          ),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16.0),
          child: ElevatedButton(
            onPressed: () => PlaceholderScreen.show(context, secured: false),
            child: Text(locale.forgotPinScreenCta),
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }

  static void show(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (c) => const ForgotPinScreen()),
    );
  }
}
