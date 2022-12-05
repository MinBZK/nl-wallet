import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../common/widget/placeholder_screen.dart';
import '../bloc/menu_bloc.dart';
import '../widget/menu_row.dart';

const _kAboutUrl = 'https://edi.pleio.nl/';

class MenuAboutPage extends StatelessWidget {
  const MenuAboutPage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return WillPopScope(
      onWillPop: () async {
        context.read<MenuBloc>().add(MenuBackPressed());
        return false;
      },
      child: ListView(
        children: [
          const SizedBox(height: 16),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16.0),
            child: Text(
              locale.menuAboutPageTitle,
              style: Theme.of(context).textTheme.bodyText1?.copyWith(fontWeight: FontWeight.bold),
            ),
          ),
          const SizedBox(height: 8),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16.0),
            child: _buildDescription(context),
          ),
          const SizedBox(height: 16),
          const Divider(height: 1),
          MenuRow(
            label: locale.menuAboutPagePrivacyCta,
            onTap: () => PlaceholderScreen.show(context, locale.menuAboutPagePrivacyCta),
          ),
          const Divider(height: 1),
          MenuRow(
            label: locale.menuAboutPageTermsCta,
            onTap: () => PlaceholderScreen.show(context, locale.menuAboutPageTermsCta),
          ),
          const Divider(height: 1),
          MenuRow(
            label: locale.menuAboutPageFeedbackCta,
            onTap: () => PlaceholderScreen.show(context, locale.menuAboutPageFeedbackCta),
          ),
          const Divider(height: 1),
        ],
      ),
    );
  }

  Widget _buildDescription(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final textStyle = Theme.of(context).textTheme.bodyText1;
    final fullText = locale.menuAboutPageDescription;
    final url = locale.menuAboutPageUrl;

    final startIndexOfUrl = fullText.indexOf(url);
    // Make sure the text still renders, albeit without the clickable url, if the translation requirement is not met.
    if (startIndexOfUrl < 0) return Text(locale.menuAboutPageDescription, style: textStyle);
    final endIndexOfUrl = startIndexOfUrl + url.length;

    return RichText(
      text: TextSpan(
        style: textStyle,
        children: [
          TextSpan(text: fullText.substring(0, startIndexOfUrl)),
          TextSpan(
            text: url,
            style: textStyle?.copyWith(
              color: Theme.of(context).primaryColor,
              decoration: TextDecoration.underline,
              decorationColor: Theme.of(context).primaryColor,
            ),
            recognizer: TapGestureRecognizer()
              ..onTap = () => launchUrlString(_kAboutUrl, mode: LaunchMode.externalApplication),
          ),
          TextSpan(text: fullText.substring(endIndexOfUrl)),
        ],
      ),
    );
  }
}
