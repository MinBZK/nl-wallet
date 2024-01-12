import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../util/extension/build_context_extension.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/version_text.dart';
import '../menu/widget/menu_row.dart';

const _kAboutUrl = 'https://edi.pleio.nl/';

class AboutScreen extends StatelessWidget {
  const AboutScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('aboutScreen'),
      body: CustomScrollView(
        slivers: [
          SliverWalletAppBar(title: context.l10n.aboutScreenTitle),
          _buildContentSliver(context),
        ],
      ),
    );
  }

  Widget _buildContentSliver(BuildContext context) {
    return SliverList.list(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: _buildDescription(context),
        ),
        const SizedBox(height: 16),
        const Divider(height: 1),
        MenuRow(
          label: context.l10n.aboutScreenPrivacyCta,
          onTap: () => PlaceholderScreen.show(context, secured: false),
        ),
        const Divider(height: 1),
        MenuRow(
          label: context.l10n.aboutScreenTermsCta,
          onTap: () => PlaceholderScreen.show(context, secured: false),
        ),
        const Divider(height: 1),
        const Padding(
          padding: EdgeInsets.all(16),
          child: VersionText(),
        ),
      ],
    );
  }

  Widget _buildDescription(BuildContext context) {
    final textStyle = context.textTheme.bodyLarge;
    final fullText = context.l10n.aboutScreenDescription;
    final url = context.l10n.aboutScreenUrl;

    final startIndexOfUrl = fullText.indexOf(url);
    // Make sure the text still renders, albeit without the clickable url, if the translation requirement is not met.
    if (startIndexOfUrl < 0) return Text(context.l10n.aboutScreenDescription, style: textStyle);
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
