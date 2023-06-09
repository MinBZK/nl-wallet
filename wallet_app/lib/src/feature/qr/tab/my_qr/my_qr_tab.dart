import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../common/widget/explanation_sheet.dart';
import '../../../common/widget/utility/max_brightness.dart';
import '../../../common/widget/button/text_icon_button.dart';

const _kLandscapeQrSize = 200.0;

class MyQrTab extends StatelessWidget {
  const MyQrTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaxBrightness(
      child: ListView(
        children: [
          Container(
            alignment: Alignment.center,
            padding: const EdgeInsets.fromLTRB(16, 24, 16, 16),
            height: context.isLandscape ? _kLandscapeQrSize : null,
            child: QrImageView(
              padding: EdgeInsets.zero,
              data: '{"id": ${DateTime.now().millisecondsSinceEpoch}',
              eyeStyle: QrEyeStyle(
                color: Theme.of(context).primaryColorDark,
                eyeShape: QrEyeShape.square,
              ),
              dataModuleStyle: QrDataModuleStyle(
                color: Theme.of(context).primaryColorDark,
                dataModuleShape: QrDataModuleShape.square,
              ),
            ),
          ),
          TextIconButton(
            child: Text(AppLocalizations.of(context).qrMyCodeTabHowToCta),
            onPressed: () => _showHowToSheet(context),
          ),
          const SizedBox(height: 16),
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
