import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../../../../../environment.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../common/sheet/explanation_sheet.dart';
import '../../../common/widget/button/button_content.dart';
import '../../../common/widget/button/tertiary_button.dart';
import '../../../common/widget/utility/max_brightness.dart';

const _kLandscapeQrSize = 200.0;

class MyQrTab extends StatelessWidget {
  const MyQrTab({super.key});

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
              data: '{"id": ${Environment.isTest ? 'test' : DateTime.now().millisecondsSinceEpoch}',
              eyeStyle: QrEyeStyle(
                color: context.theme.primaryColorDark,
                eyeShape: QrEyeShape.square,
              ),
              dataModuleStyle: QrDataModuleStyle(
                color: context.theme.primaryColorDark,
                dataModuleShape: QrDataModuleShape.square,
              ),
            ),
          ),
          TertiaryButton(
            text: Text.rich(context.l10n.qrMyCodeTabHowToCta.toTextSpan(context)),
            iconPosition: IconPosition.end,
            onPressed: () => _showHowToSheet(context),
          ),
          const SizedBox(height: 16),
        ],
      ),
    );
  }

  void _showHowToSheet(BuildContext context) {
    ExplanationSheet.show(
      context,
      title: context.l10n.qrMyCodeTabHowToSheetTitle,
      description: context.l10n.qrMyCodeTabHowToSheetDescription,
      closeButtonText: context.l10n.qrMyCodeTabHowToSheetCloseCta,
    );
  }
}
