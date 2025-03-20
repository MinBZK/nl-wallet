import 'package:flutter/material.dart';
import 'package:flutter_svg/svg.dart';
import 'package:visibility_detector/visibility_detector.dart';

import '../../../../theme/base_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../../wallet_assets.dart';
import '../../../../wallet_constants.dart';

@visibleForTesting
const scanQrButtonSize = Size(240, 228);
const _qrButtonRadius = 44.0;
const _ctaMinHeight = 28.0;

class ScanQrButton extends StatefulWidget {
  final VoidCallback onPressed;

  const ScanQrButton({required this.onPressed, super.key});

  @override
  State<ScanQrButton> createState() => _ScanQrButtonState();
}

class _ScanQrButtonState extends State<ScanQrButton> {
  final Key _visibilityDetectorKey = GlobalKey();

  late WidgetStatesController _statesController;

  /// Whether the asset & cta are visible. Relevant for PVW-4212.
  bool _isVisible = true;

  set isVisible(bool value) {
    if (_isVisible == value) return;
    _isVisible = value;
    if (mounted) setState(() {});
  }

  @override
  void initState() {
    super.initState();
    _statesController = WidgetStatesController();
    WidgetsBinding.instance.addPostFrameCallback((_) => _statesController.addListener(_updateState));
  }

  void _updateState() => setState(() {});

  @override
  void dispose() {
    _statesController.removeListener(_updateState);
    _statesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ExcludeSemantics(
      excluding: !_isVisible,
      child: Semantics(
        attributedLabel: context.l10n.scanQrButtonCta.toAttributedString(context),
        button: true,
        excludeSemantics: true /* exclude child semantics */,
        child: SizedBox.fromSize(
          size: scanQrButtonSize,
          child: Material(
            color: _statesController.value.isPressedOrFocused ? context.theme.focusColor : Colors.transparent,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(_qrButtonRadius),
              side: context.theme.elevatedButtonTheme.style?.side?.resolve(_statesController.value) ?? BorderSide.none,
            ),
            clipBehavior: Clip.hardEdge,
            child: InkWell(
              canRequestFocus: _isVisible,
              onTap: widget.onPressed,
              statesController: _statesController,
              focusColor:
                  Colors.transparent /* focus color set above so that it behaves correctly on navigation (and back) */,
              child: Center(
                child: VisibilityDetector(
                  onVisibilityChanged: (VisibilityInfo info) => isVisible = info.visibleFraction > 0.0,
                  key: _visibilityDetectorKey,
                  child: Column(
                    spacing: 8,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      AnimatedCrossFade(
                        firstChild: SvgPicture.asset(WalletAssets.svg_qr_button),
                        secondChild: SvgPicture.asset(_resolveFocusedAsset()),
                        crossFadeState: _statesController.value.isPressedOrFocused
                            ? CrossFadeState.showSecond
                            : CrossFadeState.showFirst,
                        duration: kDefaultAnimationDuration,
                      ),
                      ConstrainedBox(
                        constraints: const BoxConstraints(minHeight: _ctaMinHeight),
                        child: Center(
                          child: Text(
                            context.l10n.scanQrButtonCta,
                            textAlign: TextAlign.center,
                            style: context.textTheme.labelLarge?.copyWith(
                              decoration: _statesController.value.isFocused ? TextDecoration.underline : null,
                              color: context.theme.textButtonTheme.style?.foregroundColor
                                  ?.resolve(_statesController.value),
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  String _resolveFocusedAsset() =>
      context.isDeviceInDarkMode ? WalletAssets.svg_qr_button_focused_dark : WalletAssets.svg_qr_button_focused;
}
