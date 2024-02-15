import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

// The tappable size of the button
const _kButtonTapSize = 48.0;
// The visual size of the button
const _kButtonSize = 32.0;

class RoundedBackButton extends StatelessWidget {
  const RoundedBackButton({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: _kButtonTapSize,
      height: _kButtonTapSize,
      child: Material(
        color: Colors.transparent,
        clipBehavior: Clip.antiAlias,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(24),
        ),
        child: Semantics(
          key: const Key('introductionBackCta'),
          button: true,
          tooltip: context.l10n.generalWCAGBack,
          child: InkWell(
            onTap: () => Navigator.maybePop(context),
            child: Center(
              child: _buildVisibleButton(context),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildVisibleButton(BuildContext context) {
    return Container(
      height: _kButtonSize,
      width: _kButtonSize,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: context.colorScheme.background,
      ),
      child: Icon(
        Icons.arrow_back,
        color: context.colorScheme.onBackground,
      ),
    );
  }
}
