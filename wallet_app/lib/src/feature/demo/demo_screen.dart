import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/launch_util.dart';
import '../common/widget/bullet_list.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/utility/scroll_offset_provider.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

const _kMoreInfoUrl = 'https://edi.pleio.nl/page/view/9ed951b8-86f3-4043-b552-beda5594e9bd/publieke-nl-wallet';

class DemoScreen extends StatelessWidget {
  const DemoScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return ScrollOffsetProvider(
      child: Scaffold(
        appBar: WalletAppBar(
          title: TitleText(context.l10n.demoScreenTitle),
          actions: const [HelpIconButton()],
        ),
        body: SafeArea(
          child: LayoutBuilder(
            builder: (context, constraints) {
              return WalletScrollbar(
                child: SingleChildScrollView(
                  child: ConstrainedBox(
                    constraints: BoxConstraints(minHeight: constraints.maxHeight),
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        _buildContentSection(context),
                        _buildBottomSection(context),
                      ],
                    ),
                  ),
                ),
              );
            },
          ),
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return ConfirmButtons(
      flipVertical: true,
      primaryButton: PrimaryButton(
        text: Text(context.l10n.demoScreenContinueCta),
        onPressed: () => Navigator.restorablePushReplacementNamed(context, WalletRoutes.introductionRoute),
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.demoScreenMoreInfoCta),
        icon: const Icon(Icons.north_east_outlined),
        onPressed: () => launchUrlStringCatching(_kMoreInfoUrl),
      ),
    );
  }

  Widget _buildContentSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          TitleText(context.l10n.demoScreenTitle),
          const SizedBox(height: 8),
          BodyText(context.l10n.demoScreenContentHeader),
          const SizedBox(height: 8),
          BulletList(items: context.l10n.demoScreenContentBullets.split('\n')),
          const SizedBox(height: 8),
          BodyText(context.l10n.demoScreenContentFooter),
        ],
      ),
    );
  }
}
