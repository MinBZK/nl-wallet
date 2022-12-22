import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../wallet_constants.dart';
import '../../pin/widget/pin_keyboard.dart';

/// Force highest res version here, avoids bloating the assets with files that are temporary by nature.
const _kDigidLogoPath = 'assets/images/3.0x/digid_logo.png';

class DigidPinPage extends StatelessWidget {
  final int selectedIndex;
  final Function(int)? onKeyPressed;
  final VoidCallback? onBackspacePressed;

  const DigidPinPage({
    required this.selectedIndex,
    required this.onKeyPressed,
    required this.onBackspacePressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.max,
          children: [
            const SizedBox(height: 32),
            Center(
              child: Image.asset(
                _kDigidLogoPath,
                scale: 0.7,
              ),
            ),
            const SizedBox(height: 32),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  Expanded(
                    child: Text(
                      AppLocalizations.of(context).mockDigidScreenEnterPin,
                      style: Theme.of(context).textTheme.bodyText2,
                    ),
                  ),
                  const Icon(Icons.help, size: 20),
                ],
              ),
            ),
            const Spacer(),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: List.generate(5, (index) {
                  return _buildPinField(context, index == selectedIndex, index < selectedIndex);
                }),
              ),
            ),
            const Spacer(),
            Center(
              child: Text(
                AppLocalizations.of(context).mockDigidScreenForgotPinCta,
                style: Theme.of(context).textTheme.bodyText2?.copyWith(
                      color: Theme.of(context).primaryColor,
                      fontWeight: FontWeight.bold,
                      decoration: TextDecoration.underline,
                    ),
              ),
            ),
            const SizedBox(height: 16),
            const Divider(height: 1),
            PinKeyboard(
              onKeyPressed: onKeyPressed,
              onBackspacePressed: onBackspacePressed,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPinField(BuildContext context, bool selected, bool filled) {
    return AnimatedContainer(
      duration: kDefaultAnimationDuration,
      height: 60,
      width: 60,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: Colors.grey.withOpacity(selected || filled ? 0.01 : 0.4),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: Colors.grey, width: 2),
      ),
      child: filled ? Text('*', style: Theme.of(context).textTheme.headline2) : const SizedBox.shrink(),
    );
  }
}
