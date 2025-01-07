// coverage:ignore-file
import 'dart:math';

import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../../wallet_constants.dart';
import '../../pin/widget/pin_keyboard.dart';

const _kDigidPinCount = 5;

class DigidPinPage extends StatelessWidget {
  final int selectedIndex;
  final Function(int)? onKeyPressed;
  final VoidCallback? onBackspacePressed;

  const DigidPinPage({
    required this.selectedIndex,
    required this.onKeyPressed,
    required this.onBackspacePressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: OrientationBuilder(
        builder: (context, orientation) {
          switch (orientation) {
            case Orientation.portrait:
              return _buildPortrait(context);
            case Orientation.landscape:
              return _buildLandscape(context);
          }
        },
      ),
    );
  }

  Widget _buildPortrait(BuildContext context) {
    return SafeArea(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.max,
        children: [
          const SizedBox(height: 32),
          Center(
            child: Image.asset(
              WalletAssets.logo_digid_large,
              scale: 0.7,
            ),
          ),
          const SizedBox(height: 32),
          _buildEnterPinInfoSection(context),
          const Spacer(),
          _buildPinSection(context),
          const Spacer(),
          _buildForgotPinCta(context),
          const SizedBox(height: 16),
          const Divider(height: 1),
          PinKeyboard(
            onKeyPressed: onKeyPressed,
            onBackspacePressed: onBackspacePressed,
          ),
        ],
      ),
    );
  }

  Widget _buildLandscape(BuildContext context) {
    return SafeArea(
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisAlignment: MainAxisAlignment.center,
              mainAxisSize: MainAxisSize.max,
              children: [
                const Spacer(),
                Center(
                  child: Image.asset(
                    WalletAssets.logo_digid_large,
                    scale: 0.7,
                  ),
                ),
                const SizedBox(height: 32),
                _buildEnterPinInfoSection(context),
                const Spacer(),
                _buildPinSection(context),
                const Spacer(),
                _buildForgotPinCta(context),
                const Spacer(),
              ],
            ),
          ),
          Expanded(
            child: PinKeyboard(
              onKeyPressed: onKeyPressed,
              onBackspacePressed: onBackspacePressed,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEnterPinInfoSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Expanded(
            child: Text(
              context.l10n.mockDigidScreenEnterPin,
              style: context.textTheme.bodyMedium,
            ),
          ),
          const Icon(Icons.help, size: 20),
        ],
      ),
    );
  }

  Widget _buildPinSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: List.generate(_kDigidPinCount, (index) {
          return _buildPinField(context, index == selectedIndex, index < selectedIndex);
        }),
      ),
    );
  }

  Widget _buildForgotPinCta(BuildContext context) {
    return Center(
      child: ExcludeSemantics(
        child: Text(
          context.l10n.mockDigidScreenForgotPinCta,
          style: context.textTheme.bodyMedium?.copyWith(
            color: context.colorScheme.primary,
            fontWeight: FontWeight.bold,
            decoration: TextDecoration.underline,
          ),
        ),
      ),
    );
  }

  Widget _buildPinField(BuildContext context, bool selected, bool filled) {
    final maxWidth = (context.mediaQuery.size.width - 32) / _kDigidPinCount;
    return AnimatedContainer(
      duration: kDefaultAnimationDuration,
      height: min(60, maxWidth),
      width: min(60, maxWidth),
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: Colors.grey.withValues(alpha: selected || filled ? 0.01 : 0.4),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: Colors.grey, width: 2),
      ),
      child: filled ? Text('*', style: context.textTheme.displayMedium) : const SizedBox.shrink(),
    );
  }
}
