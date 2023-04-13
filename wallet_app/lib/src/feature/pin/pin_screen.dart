import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import 'pin_page.dart';

class PinScreen extends StatelessWidget {
  final VoidCallback? onUnlock;

  const PinScreen({this.onUnlock, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).pinScreenTitle),
        leading: const SizedBox.shrink(),
      ),
      body: PinPage(onPinValidated: onUnlock),
    );
  }
}
