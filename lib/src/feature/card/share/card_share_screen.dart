import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../../common/widget/explanation_sheet.dart';
import '../../common/widget/max_brightness.dart';
import '../../common/widget/text_icon_button.dart';

class CardShareScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) screenTitle when opening the CardShareScreen');
    }
  }

  final String screenTitle;

  const CardShareScreen({required this.screenTitle, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(screenTitle),
      ),
      body: MaxBrightness(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
              child: Text(locale.cardShareScreenSubtitle, style: theme.textTheme.bodyText1),
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
              child: QrImage(
                padding: EdgeInsets.zero,
                data: '{"id": ${DateTime.now().millisecondsSinceEpoch}',
                foregroundColor: theme.primaryColorDark,
              ),
            ),
            TextIconButton(
              child: Text(locale.cardShareScreenExplanationCta),
              onPressed: () => _showHowToSheet(context),
            ),
          ],
        ),
      ),
    );
  }

  void _showHowToSheet(BuildContext context) {
    final locale = AppLocalizations.of(context);
    ExplanationSheet.show(
      context,
      title: locale.cardShareScreenExplanationSheetTitle,
      description: locale.cardShareScreenExplanationSheetDescription,
      closeButtonText: locale.cardShareScreenExplanationSheetCloseCta,
    );
  }
}
