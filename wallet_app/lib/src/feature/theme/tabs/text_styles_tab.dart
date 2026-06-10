import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../environment.dart';
import '../../../data/service/navigation_service.dart';
import '../../../data/service/work_manager_service.dart';
import '../../../domain/model/navigation/navigation_request.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/spacer/paragraph_spacer.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../disclosure/argument/disclosure_screen_argument.dart';
import '../../issuance/argument/issuance_screen_argument.dart';
import '../../notification/debug/scheduled_notifications_screen.dart';
import '../../sign/argument/sign_screen_argument.dart';
import '../theme_screen.dart';

class TextStylesTab extends StatelessWidget {
  const TextStylesTab({super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      children: [
        ..._buildMaterialTextStyles(context),
        const Divider(height: 24),
        ..._buildWalletTextStyles(context),
        ..._buildDebugPages(context),
        ..._buildScenarios(context),
      ],
    );
  }

  List<Widget> _buildMaterialTextStyles(BuildContext context) {
    return [
      // Display
      Text('DisplayLarge', style: context.textTheme.displayLarge),
      Text('DisplayMedium', style: context.textTheme.displayMedium),
      Text('DisplaySmall', style: context.textTheme.displaySmall),
      // Headline
      Text('HeadlineLarge', style: context.textTheme.headlineLarge),
      Text('HeadlineMedium', style: context.textTheme.headlineMedium),
      Text('HeadlineSmall', style: context.textTheme.headlineSmall),
      // Title
      Text('TitleLarge', style: context.textTheme.titleLarge),
      Text('TitleMedium', style: context.textTheme.titleMedium),
      Text('TitleSmall', style: context.textTheme.titleSmall),
      // Body
      Text('BodyLarge', style: context.textTheme.bodyLarge),
      Text('BodyMedium', style: context.textTheme.bodyMedium),
      Text('BodySmall', style: context.textTheme.bodySmall),
      // Label
      Text('LabelLarge', style: context.textTheme.labelLarge),
      Text('LabelMedium', style: context.textTheme.labelMedium),
      Text('LabelSmall', style: context.textTheme.labelSmall),
    ];
  }

  List<Widget> _buildWalletTextStyles(BuildContext context) {
    return [
      const TitleText('Title Text'),
      const ParagraphSpacer(),
      const BodyText('Body Text'),
    ];
  }

  List<Widget> _buildDebugPages(BuildContext context) {
    return [
      const ThemeSectionHeader(title: 'Debug Pages'),
      const SizedBox(height: 12),
      SecondaryButton(
        onPressed: () => ScheduledNotificationsScreen.show(context),
        icon: const Icon(Icons.notifications_outlined),
        text: const Text('Notifications'),
      ),
      const SizedBox(height: 12),
      SecondaryButton(
        onPressed: () => performRevocationCheckTask(initCore: false),
        icon: const Icon(Icons.sync),
        text: const Text('Force FRB Background Sync'),
      ),
      const SizedBox(height: 12),
    ];
  }

  List<Widget> _buildScenarios(BuildContext context) {
    if (!Environment.mockRepositories) return [];
    return [
      const ThemeSectionHeader(title: 'Scenarios'),
      const SizedBox(height: 12),
      ..._buildDisclosureScenarios(context),
      ..._buildIssuanceScenarios(context),
      ..._buildSigningScenarios(context),
      ..._buildTransferScenarios(context),
    ];
  }

  List<Widget> _buildDisclosureScenarios(BuildContext context) {
    return [
      SecondaryButton(
        onPressed: () {
          final request = NavigationRequest.disclosure(
            argument: const DisclosureScreenArgument(
              type: .remote(
                'walletdebuginteraction://deeplink#%7B%22id%22%3A%22OPEN_BANK_ACCOUNT%22%2C%22type%22%3A%22verify%22%7D',
                isQrCode: false,
              ),
            ),
          );
          context.read<NavigationService>().handleNavigationRequest(request);
        },
        icon: const Icon(Icons.share_outlined),
        text: const Text('disclose'),
      ),
      const SizedBox(height: 12),
      SecondaryButton(
        onPressed: () {
          final request = NavigationRequest.disclosure(
            argument: const DisclosureScreenArgument(
              type: .remote(
                'walletdebuginteraction://deeplink#%7B%22id%22%3A%22AMSTERDAM_LOGIN%22%2C%22type%22%3A%22verify%22%7D',
                isQrCode: false,
              ),
            ),
          );
          context.read<NavigationService>().handleNavigationRequest(request);
        },
        icon: const Icon(Icons.login_outlined),
        text: const Text('login'),
      ),
      const SizedBox(height: 12),
      SecondaryButton(
        onPressed: () {
          final request = NavigationRequest.disclosure(
            argument: const DisclosureScreenArgument(
              type: .remote(
                'walletdebuginteraction://deeplink#%7B%22id%22%3A%22CAR_RENTAL%22%2C%22type%22%3A%22verify%22%7D',
                isQrCode: false,
              ),
            ),
          );
          context.read<NavigationService>().handleNavigationRequest(request);
        },
        icon: const Icon(Icons.warning_amber_outlined),
        text: const Text('missing attributes'),
      ),
      const SizedBox(height: 12),
    ];
  }

  List<Widget> _buildIssuanceScenarios(BuildContext context) {
    return [
      SecondaryButton(
        onPressed: () {
          final request = NavigationRequest.issuance(
            argument: const IssuanceScreenArgument(
              uri:
                  'walletdebuginteraction://deeplink#%7B%22id%22%3A%22DRIVING_LICENSE%22%2C%22type%22%3A%22issue%22%7D',
              isQrCode: false,
            ),
          );
          context.read<NavigationService>().handleNavigationRequest(request);
        },
        icon: const Icon(Icons.credit_card_outlined),
        text: const Text('issue'),
      ),
      const SizedBox(height: 12),
    ];
  }

  List<Widget> _buildSigningScenarios(BuildContext context) {
    return [
      SecondaryButton(
        onPressed: () {
          final request = NavigationRequest.sign(
            argument: const SignScreenArgument(
              uri:
                  'walletdebuginteraction://deeplink#%7B%22id%22%3A%22RENTAL_AGREEMENT%22%2C%22type%22%3A%22sign%22%7D',
            ),
          );
          context.read<NavigationService>().handleNavigationRequest(request);
        },
        icon: const Icon(Icons.document_scanner_outlined),
        text: const Text('sign'),
      ),
      const SizedBox(height: 12),
    ];
  }

  List<Widget> _buildTransferScenarios(BuildContext context) {
    return [
      SecondaryButton(
        onPressed: () =>
            context.read<NavigationService>().handleNavigationRequest(NavigationRequest.walletTransferSource('uri')),
        icon: const Icon(Icons.move_down_outlined),
        text: const Text('Wallet Transfer (Source)'),
      ),
      const SizedBox(height: 12),
      SecondaryButton(
        onPressed: () => Navigator.pushNamed(context, WalletRoutes.walletTransferTargetRoute),
        icon: const Icon(Icons.move_up_outlined),
        text: const Text('Wallet Transfer (Target)'),
      ),
      const SizedBox(height: 12),
    ];
  }
}
