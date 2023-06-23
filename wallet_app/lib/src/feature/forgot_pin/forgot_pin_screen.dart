import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/placeholder_screen.dart';

const _kPinHeaderImage = 'assets/images/forgot_pin_header.png';

class ForgotPinScreen extends StatelessWidget {
  const ForgotPinScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.forgotPinScreenTitle),
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
    return Scrollbar(
      child: ListView(
        padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
        children: [
          Image.asset(_kPinHeaderImage, fit: BoxFit.fitWidth),
          const SizedBox(height: 24),
          Text(
            context.l10n.forgotPinScreenHeadline,
            textAlign: TextAlign.start,
            style: context.textTheme.displayMedium,
          ),
          const SizedBox(height: 8),
          Text(
            context.l10n.forgotPinScreenDescription,
            textAlign: TextAlign.start,
            style: context.textTheme.bodyLarge,
          ),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16.0),
          child: ElevatedButton(
            onPressed: () => PlaceholderScreen.show(context, secured: false),
            child: Text(context.l10n.forgotPinScreenCta),
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
