import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_constants.dart';
import 'bloc/flashlight_cubit.dart';
import 'tab/my_qr/my_qr_tab.dart';
import 'tab/qr_scan/bloc/qr_scan_bloc.dart';
import 'tab/qr_scan/qr_scan_tab.dart';
import 'widget/qr_screen_flash_toggle.dart';

class QrScreen extends StatelessWidget {
  const QrScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final tabs = [
      Tab(text: AppLocalizations.of(context).qrScreenScanTabTitle),
      Tab(text: AppLocalizations.of(context).qrScreenMyCodeTabTitle),
    ];
    return BlocProvider(
      create: (context) => FlashlightCubit(),
      child: DefaultTabController(
        length: tabs.length,
        animationDuration: kDefaultAnimationDuration,
        child: Scaffold(
          appBar: AppBar(
            title: Text(AppLocalizations.of(context).qrScreenTitle),
            bottom: TabBar(tabs: tabs),
            actions: const [QrScreenFlashToggle()],
          ),
          body: TabBarView(
            children: [
              BlocProvider(
                create: (context) => QrScanBloc(context.read()),
                child: const QrScanTab(),
              ),
              const MyQrTab(),
            ],
          ),
        ),
      ),
    );
  }
}
