import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import 'pin_page.dart';

class PinScreen extends StatelessWidget {
  final VoidCallback? onUnlock;

  const PinScreen({this.onUnlock, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.pinScreenTitle),
        leading: const SizedBox.shrink(),
      ),
      body: PinPage(onPinValidated: onUnlock),
    );
  }
}
