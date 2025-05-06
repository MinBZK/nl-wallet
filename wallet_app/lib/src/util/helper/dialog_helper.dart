import 'package:flutter/material.dart';

class DialogHelper {
  DialogHelper._();

  /// Pops all present (top-level) dialogs
  static Future<void> dismissOpenDialogs(BuildContext context) {
    final navigator = Navigator.of(context);
    return Future.microtask(() {
      navigator.popUntil((route) {
        final isDialog = route is ModalBottomSheetRoute || route is DialogRoute;
        return !isDialog;
      });
    });
  }
}
