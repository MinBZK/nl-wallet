import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/flow_progress.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/helper/onboarding_helper.dart';
import '../common/page/terminal_page.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/utility/scroll_offset_provider.dart';
import '../common/widget/wallet_app_bar.dart';
import 'bloc/revocation_code_bloc.dart';
import 'widget/revocation_code_text.dart';

class RevocationCodeScreen extends StatelessWidget {
  const RevocationCodeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return ScrollOffsetProvider(
      child: Scaffold(
        appBar: WalletAppBar(
          automaticallyImplyLeading: false,
          actions: [const HelpIconButton()],
          title: TitleText(context.l10n.revocationCodeScreenTitle),
          progress: FlowProgress(currentStep: OnboardingHelper.totalSteps - 4, totalSteps: OnboardingHelper.totalSteps),
        ),
        body: SafeArea(
          child: BlocConsumer<RevocationCodeBloc, RevocationCodeState>(
            listener: (BuildContext context, RevocationCodeState state) {
              if (state is RevocationCodeSaveSuccess) {
                Navigator.pushReplacementNamed(context, WalletRoutes.walletPersonalizeRoute);
              }
            },
            builder: (BuildContext context, RevocationCodeState state) {
              switch (state) {
                case RevocationCodeInitial():
                  return _buildLoading(context);
                case RevocationCodeLoadSuccess():
                  return _buildContent(context, state.revocationCode);
                case RevocationCodeSaveSuccess():
                  return _buildContent(context, state.revocationCode);
              }
            },
          ),
        ),
      ),
    );
  }

  Widget _buildLoading(BuildContext context) {
    return Column(
      children: [
        const SizedBox(height: 12),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: TitleText(context.l10n.revocationCodeScreenTitle),
        ),
        const Expanded(child: CenteredLoadingIndicator()),
      ],
    );
  }

  Widget _buildContent(BuildContext context, String revocationCode) {
    return TerminalPage(
      title: context.l10n.revocationCodeScreenTitle,
      description: context.l10n.revocationCodeScreenDescription,
      illustration: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: Column(
          children: [
            BodyText(context.l10n.revocationCodeScreenCodeHeader),
            const SizedBox(height: 8),
            RevocationCodeText(revocationCode: revocationCode),
            const SizedBox(height: 24),
            BodyText(context.l10n.revocationCodeScreenCodeFooter),
          ],
        ),
      ),
      primaryButtonCta: context.l10n.revocationCodeScreenContinueCta,
      onPrimaryPressed: () => context.read<RevocationCodeBloc>().add(const RevocationCodeContinuePressed()),
    );
  }
}
