import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../../../common/widget/explanation_sheet.dart';
import '../../../common/widget/text_icon_button.dart';
import '../../widget/max_brightness.dart';

class MyQrTab extends StatelessWidget {
  const MyQrTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaxBrightness(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 24, 16, 16),
            child: QrImage(
              padding: EdgeInsets.zero,
              data: '{"id": ${DateTime.now().millisecondsSinceEpoch}',
              foregroundColor: Theme.of(context).primaryColorDark,
            ),
          ),
          TextIconButton(
            child: Text(AppLocalizations.of(context).qrMyCodeTabHowToCta),
            onPressed: () => _showHowToSheet(context),
          ),
        ],
      ),
    );
  }

  void _showHowToSheet(BuildContext context) {
    final locale = AppLocalizations.of(context);
    ExplanationSheet.show(
      context,
      title: locale.qrMyCodeTabHowToSheetTitle,
      description: locale.qrMyCodeTabHowToSheetDescription,
      closeButtonText: locale.qrMyCodeTabHowToSheetCloseCta,
    );
  }
}
