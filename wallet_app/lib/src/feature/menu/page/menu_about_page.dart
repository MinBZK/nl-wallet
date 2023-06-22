import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/placeholder_screen.dart';
import '../bloc/menu_bloc.dart';
import '../widget/menu_row.dart';

const _kAboutUrl = 'https://edi.pleio.nl/';

class MenuAboutPage extends StatelessWidget {
  const MenuAboutPage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return WillPopScope(
      onWillPop: () async {
        context.read<MenuBloc>().add(MenuBackPressed());
        return false;
      },
      child: Scrollbar(
        child: ListView(
          children: [
            const SizedBox(height: 16),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Text(
                context.l10n.menuAboutPageTitle,
                style: context.textTheme.bodyLarge?.copyWith(fontWeight: FontWeight.bold),
              ),
            ),
            const SizedBox(height: 8),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildDescription(context),
            ),
            const SizedBox(height: 16),
            const Divider(height: 1),
            MenuRow(
              label: context.l10n.menuAboutPagePrivacyCta,
              onTap: () => PlaceholderScreen.show(context),
            ),
            const Divider(height: 1),
            MenuRow(
              label: context.l10n.menuAboutPageTermsCta,
              onTap: () => PlaceholderScreen.show(context),
            ),
            const Divider(height: 1),
            MenuRow(
              label: context.l10n.menuAboutPageFeedbackCta,
              onTap: () => PlaceholderScreen.show(context),
            ),
            const Divider(height: 1),
          ],
        ),
      ),
    );
  }

  Widget _buildDescription(BuildContext context) {
    final textStyle = context.textTheme.bodyLarge;
    final fullText = context.l10n.menuAboutPageDescription;
    final url = context.l10n.menuAboutPageUrl;

    final startIndexOfUrl = fullText.indexOf(url);
    // Make sure the text still renders, albeit without the clickable url, if the translation requirement is not met.
    if (startIndexOfUrl < 0) return Text(context.l10n.menuAboutPageDescription, style: textStyle);
    final endIndexOfUrl = startIndexOfUrl + url.length;

    return RichText(
      text: TextSpan(
        style: textStyle,
        children: [
          TextSpan(text: fullText.substring(0, startIndexOfUrl)),
          TextSpan(
            text: url,
            style: textStyle?.copyWith(
              color: context.colorScheme.primary,
              decoration: TextDecoration.underline,
              decorationColor: context.colorScheme.primary,
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
