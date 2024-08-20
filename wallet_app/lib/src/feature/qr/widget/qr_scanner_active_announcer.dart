import 'dart:async';

import 'package:flutter/cupertino.dart';
import 'package:flutter/rendering.dart';

import '../../../util/extension/build_context_extension.dart';

const kAnnouncementInterval = Duration(seconds: 10);

/// An invisible widget that repeatedly informs the user
/// that the scanner is active (using TalkBack/VoiceOver).
class QrScannerActiveAnnouncer extends StatefulWidget {
  final Widget? child;

  const QrScannerActiveAnnouncer({this.child, super.key});

  @override
  State<QrScannerActiveAnnouncer> createState() => _QrScannerActiveAnnouncerState();
}

class _QrScannerActiveAnnouncerState extends State<QrScannerActiveAnnouncer> {
  late StreamSubscription _subscription;

  @override
  void initState() {
    super.initState();
    final periodStream = Stream.periodic(kAnnouncementInterval);
    _subscription = periodStream.listen(_onData);
  }

  void _onData(event) {
    SemanticsService.announce(context.l10n.qrScreenScanScannerActiveWCAGAnnouncement, TextDirection.ltr);
  }

  @override
  Widget build(BuildContext context) => widget.child ?? const SizedBox.shrink();

  @override
  void dispose() {
    _subscription.cancel();
    super.dispose();
  }
}
