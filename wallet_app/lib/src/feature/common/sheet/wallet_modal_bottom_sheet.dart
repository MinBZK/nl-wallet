import 'package:flutter/material.dart';

/// Shows a modal bottom sheet whose scrim still dismisses on tap, but is hidden from assistive technology,
/// which would otherwise expose it as a meaningless "Scrim" node (also without a screen reader). See PVW-6339.
Future<T?> showWalletModalBottomSheet<T>({
  required BuildContext context,
  required WidgetBuilder builder,
  bool isDismissible = true,
}) {
  final navigator = Navigator.of(context);
  final localizations = MaterialLocalizations.of(context);
  return navigator.push(
    _WalletModalBottomSheetRoute<T>(
      builder: builder,
      capturedThemes: InheritedTheme.capture(from: context, to: navigator.context),
      barrierLabel: localizations.scrimLabel,
      barrierOnTapHint: localizations.scrimOnTapHint(localizations.bottomSheetLabel),
      modalBarrierColor: Theme.of(context).bottomSheetTheme.modalBarrierColor,
      isDismissible: isDismissible,
      isScrollControlled: true,
    ),
  );
}

class _WalletModalBottomSheetRoute<T> extends ModalBottomSheetRoute<T> {
  _WalletModalBottomSheetRoute({
    required super.builder,
    required super.capturedThemes,
    required super.barrierLabel,
    required super.barrierOnTapHint,
    required super.modalBarrierColor,
    required super.isDismissible,
    required super.isScrollControlled,
  });

  @override
  bool get semanticsDismissible => false;
}
