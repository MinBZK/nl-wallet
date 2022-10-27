import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_constants.dart';
import 'tab/my_qr_tab.dart';
import 'tab/scan_qr_tab.dart';

class QrScreen extends StatelessWidget {
  const QrScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 2,
      animationDuration: kDefaultAnimationDuration,
      child: Scaffold(
        appBar: AppBar(
          title: Text(AppLocalizations.of(context).qrScreenTitle),
          bottom: TabBar(
            tabs: [
              Tab(text: AppLocalizations.of(context).qrScreenMyCodeTabTitle),
              Tab(text: AppLocalizations.of(context).qrScreenScanTabTitle),
            ],
          ),
        ),
        body: const TabBarView(
          children: [
            MyQrTab(),
            ScanQrTab(),
          ],
        ),
      ),
    );
  }
}
