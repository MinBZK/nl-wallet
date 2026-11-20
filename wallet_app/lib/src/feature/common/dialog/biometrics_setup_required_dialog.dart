import 'package:app_settings/app_settings.dart';
import 'package:material_ui/material_ui.dart';

import '../../../domain/usecase/biometrics/biometrics.dart';
import '../../../util/extension/biometrics_extension.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/text/dialog_title_text.dart';

class BiometricsSetupRequiredDialog extends StatelessWidget {
  final Biometrics biometrics;

  const BiometricsSetupRequiredDialog({required this.biometrics, super.key});

  @override
  Widget build(BuildContext context) {
    final supportedBiometricsText = biometrics.prettyPrint(context);
    return AlertDialog(
      scrollable: true,
      title: DialogTitleText(context.l10n.biometricSettingsScreenSetupDialogTitle(supportedBiometricsText)),
      content: Text.rich(
        context.l10n.biometricSettingsScreenSetupDialogDescription(supportedBiometricsText).toTextSpan(context),
      ),
      actions: <Widget>[
        TextButton(
          child: Text.rich(context.l10n.generalDialogCloseCta.toTextSpan(context)),
          onPressed: () => Navigator.of(context).pop(),
        ),
        TextButton(
          child: Text.rich(context.l10n.biometricSettingsScreenSetupDialogOpenSettingsCta.toTextSpan(context)),
          onPressed: () {
            // Generic settings: plugins that open the biometric settings directly don't work on e.g. a Pixel 6 Pro.
            AppSettings.openAppSettings(type: AppSettingsType.device, asAnotherTask: true);
            Navigator.of(context).pop();
          },
        ),
      ],
    );
  }

  static Future<void> show(BuildContext context, Biometrics biometrics) {
    return showDialog<void>(
      context: context,
      builder: (BuildContext context) => BiometricsSetupRequiredDialog(biometrics: biometrics),
    );
  }
}
