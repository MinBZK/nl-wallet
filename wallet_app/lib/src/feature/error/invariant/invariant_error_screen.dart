import 'dart:io';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:sentry_flutter/sentry_flutter.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/bullet_list.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/tertiary_button.dart';
import '../../common/widget/page_illustration.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../common/widget/wallet_scrollbar.dart';
import 'argument/invariant_error_screen_argument.dart';
import 'invariant_error_details_sheet.dart';

class InvariantErrorScreen extends StatelessWidget {
  static InvariantErrorScreenArgument? getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return InvariantErrorScreenArgument.fromJson(args! as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      return null;
    }
  }

  /// Technical error code/details, surfaced through the details sheet.
  final String? code;

  const InvariantErrorScreen({
    this.code,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.invariantErrorScreenTitle),
        automaticallyImplyLeading: false,
        actions: const [HelpIconButton()],
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: SingleChildScrollView(
                  child: _buildContentSection(context),
                ),
              ),
            ),
            _buildBottomSection(context),
          ],
        ),
      ),
    );
  }

  Widget _buildContentSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        children: [
          const SizedBox(height: 12),
          TitleText(context.l10n.invariantErrorScreenTitle),
          const SizedBox(height: 8),
          BodyText(context.l10n.invariantErrorScreenDescription),
          const SizedBox(height: 24),
          BodyText(context.l10n.invariantErrorScreenWhatCanYouDoTitle),
          BulletList(items: context.l10n.invariantErrorScreenWhatCanYouDoBullets.split('\n')),
          const SizedBox(height: 24),
          BodyText(context.l10n.invariantErrorScreenStillNotWorkingTitle),
          BulletList(items: context.l10n.invariantErrorScreenStillNotWorkingBullets.split('\n')),
          const SizedBox(height: 24),
          BodyText(context.l10n.invariantErrorScreenHelpdesk),
          const SizedBox(height: 24),
          const PageIllustration(asset: WalletAssets.svg_error_general, padding: EdgeInsets.zero),
          const SizedBox(height: 24),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Divider(),
        ConfirmButtons(
          primaryButton: PrimaryButton(
            text: Text.rich(context.l10n.invariantErrorScreenStartAgainCta.toTextSpan(context)),
            icon: const Icon(Icons.replay_outlined),
            onPressed: () async {
              if (Sentry.isEnabled) {
                await Sentry.close();
              }
              // Crash to force a clean restart; the core is in an unrecoverable state.
              // TODO(PVW-5914): replace with a proper wallet_core/app restart once the spike concludes.
              exit(1);
            },
          ),
          secondaryButton: TertiaryButton(
            text: Text.rich(context.l10n.invariantErrorScreenSeeDetailsCta.toTextSpan(context)),
            icon: const Icon(Icons.info_outline_rounded),
            onPressed: () => InvariantErrorDetailsSheet.show(context, code: code),
          ),
          forceVertical: !context.isLandscape,
          flipVertical: true,
        ),
      ],
    );
  }
}
