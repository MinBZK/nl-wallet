import 'dart:async';
import 'dart:math';

import 'package:flutter/material.dart';

import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';

class PinTimeoutDescription extends StatefulWidget {
  final DateTime expiryTime;
  final VoidCallback? onExpire;

  const PinTimeoutDescription({
    required this.expiryTime,
    this.onExpire,
    super.key,
  });

  @override
  State<PinTimeoutDescription> createState() => _PinTimeoutDescriptionState();
}

class _PinTimeoutDescriptionState extends State<PinTimeoutDescription> with SingleTickerProviderStateMixin {
  late Timer _timer;

  @override
  void initState() {
    super.initState();
    _timer = Timer.periodic(
      const Duration(seconds: 1),
      (Timer t) => setState(_checkExpiry),
    );
  }

  @override
  void dispose() {
    _timer.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final timeLeft = _generateTimeLeft(context); // bold text
    final fullText = context.l10n.pinTimeoutScreenTimeoutCountdown(timeLeft);
    final regularTextParts = fullText.split(timeLeft); // length == 2 (before and after bold text == timeLeft)

    return Text.rich(
      TextSpan(
        style: context.textTheme.bodyLarge,
        text: regularTextParts.first,
        children: [
          TextSpan(
            text: timeLeft,
            style: context.textTheme.bodyLarge?.copyWith(fontVariations: [BaseWalletTheme.fontVariationBold]),
          ),
          TextSpan(text: regularTextParts.last),
        ],
      ),
      style: context.textTheme.bodyLarge,
      textAlign: TextAlign.start,
    );
  }

  String _generateTimeLeft(BuildContext context) {
    final diff = widget.expiryTime.difference(DateTime.now());
    if (diff.inSeconds > 60) {
      return context.l10n.generalMinutes(diff.inMinutes);
    } else {
      return context.l10n.generalSeconds(max(0, diff.inSeconds));
    }
  }

  void _checkExpiry() {
    final diff = widget.expiryTime.difference(DateTime.now());
    if (diff.inSeconds == 0 || diff.isNegative) widget.onExpire?.call();
  }
}
