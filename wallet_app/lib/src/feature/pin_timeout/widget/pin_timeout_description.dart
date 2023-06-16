import 'dart:async';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import 'package:flutter/material.dart';

import '../../../util/extension/string_extension.dart';

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
          (Timer t) =>
          setState(() => _checkExpiry()),
    );
  }

  @override
  void dispose() {
    _timer.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Text.rich(
      TextSpan(
          text: locale.pinTimeoutScreenTimeoutPrefix.addSpaceSuffix,
          children: [
            TextSpan(
              text: _generateTimeLeft(context),
              style: Theme
                  .of(context)
                  .textTheme
                  .bodyLarge
                  ?.copyWith(fontWeight: FontWeight.bold),
            ),
          ]),
      style: Theme
          .of(context)
          .textTheme
          .bodyLarge,
      textAlign: TextAlign.start,
    );
  }

  String _generateTimeLeft(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final diff = widget.expiryTime.difference(DateTime.now());
    if (diff.inSeconds > 60) {
      return locale.generalMinutes(diff.inMinutes);
    } else {
      return locale.generalSeconds(diff.inSeconds);
    }
  }

  void _checkExpiry() {
    final diff = widget.expiryTime.difference(DateTime.now());
    if (diff.inSeconds == 0 || diff.isNegative) widget.onExpire?.call();
  }
}
