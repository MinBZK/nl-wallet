import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_constants.dart';
import '../../pin/widget/pin_field.dart';
import '../../pin/widget/pin_keyboard.dart';

class SetupSecurityPinPage extends StatelessWidget {
  final String title;
  final Function(int)? onKeyPressed;
  final VoidCallback? onBackspacePressed;
  final int enteredDigits;
  final bool showInput;
  final bool isShowingError;

  const SetupSecurityPinPage({
    required this.title,
    required this.onKeyPressed,
    required this.onBackspacePressed,
    required this.enteredDigits,
    this.showInput = true,
    this.isShowingError = false,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return OrientationBuilder(builder: (context, orientation) {
      switch (orientation) {
        case Orientation.portrait:
          return _buildPortrait(context);
        case Orientation.landscape:
          return _buildLandscape(context);
      }
    });
  }

  Widget _buildPortrait(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      mainAxisAlignment: MainAxisAlignment.start,
      children: [
        Expanded(
          child: Scrollbar(
            thumbVisibility: true,
            trackVisibility: true,
            child: SingleChildScrollView(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
              child: Align(
                alignment: Alignment.topLeft,
                child: Text(
                  title,
                  style: context.textTheme.displayMedium,
                ),
              ),
            ),
          ),
        ),
        _buildPinField(),
        const SizedBox(height: 16),
        Visibility(
          visible: showInput,
          maintainSize: true,
          maintainAnimation: true,
          maintainState: true,
          child: PinKeyboard(
            onKeyPressed: onKeyPressed,
            onBackspacePressed: onBackspacePressed,
          ),
        ),
      ],
    );
  }

  Widget _buildLandscape(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: Scrollbar(
            trackVisibility: true,
            thumbVisibility: true,
            child: SingleChildScrollView(
              padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 38),
              child: Align(
                alignment: Alignment.centerLeft,
                child: Text(
                  title,
                  style: context.textTheme.displayMedium,
                  textAlign: TextAlign.start,
                ),
              ),
            ),
          ),
        ),
        if (showInput)
          Expanded(
            child: Column(
              children: [
                const SizedBox(height: 16),
                _buildPinField(),
                Expanded(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(vertical: 16),
                    child: PinKeyboard(
                      onKeyPressed: onKeyPressed,
                      onBackspacePressed: onBackspacePressed,
                    ),
                  ),
                ),
              ],
            ),
          ),
      ],
    );
  }

  Widget _buildPinField() {
    return PinField(
      digits: kPinDigits,
      enteredDigits: enteredDigits,
      state: isShowingError ? PinFieldState.error : PinFieldState.idle,
    );
  }
}
