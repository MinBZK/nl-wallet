import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/flow_progress.dart';
import '../../domain/usecase/revocation/get_revocation_code_usecase.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/page/terminal_page.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/pin_header.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/utility/scroll_offset_provider.dart';
import '../common/widget/wallet_app_bar.dart';
import '../pin/bloc/pin_bloc.dart';
import '../pin/pin_page.dart';
import '../revocation/widget/revocation_code_text.dart';
import 'bloc/review_revocation_code_bloc.dart';

class ReviewRevocationCodeScreen extends StatelessWidget {
  const ReviewRevocationCodeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return ScrollOffsetProvider(
      child: Scaffold(
        appBar: WalletAppBar(
          automaticallyImplyLeading: false,
          leading: _buildBackButton(context),
          actions: [const HelpIconButton(), _buildCloseButton(context)],
          title: TitleText(context.l10n.reviewRevocationCodeScreenTitle),
          progress: _resolveProgress(context),
        ),
        body: SafeArea(
          child: BlocBuilder<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
            builder: (context, state) {
              Widget content;
              switch (state) {
                case ReviewRevocationCodeInitial():
                  content = _buildInitial(context);
                case ReviewRevocationCodeProvidePin():
                  content = _buildPin(context);
                case ReviewRevocationCodeSuccess():
                  content = _buildSuccess(context, state.revocationCode);
              }
              final animateBackwards = state is ReviewRevocationCodeInitial;
              return FakePagingAnimatedSwitcher(
                animateBackwards: animateBackwards,
                child: content,
              );
            },
          ),
        ),
      ),
    );
  }

  Widget _buildInitial(BuildContext context) {
    return TerminalPage(
      title: context.l10n.reviewRevocationCodeScreenTitle,
      description: context.l10n.reviewRevocationCodeScreenDescription,
      illustration: const PageIllustration(asset: WalletAssets.svg_security_code),
      primaryButtonCta: context.l10n.reviewRevocationCodeScreenViewCta,
      onPrimaryPressed: () => context.read<ReviewRevocationCodeBloc>().add(const ReviewRevocationCodeRequested()),
      primaryButtonIcon: const Icon(Icons.arrow_forward_outlined),
      secondaryButtonCta: context.l10n.generalBottomBackCta,
      secondaryButtonIcon: const Icon(Icons.arrow_back_outlined),
      onSecondaryButtonPressed: () => Navigator.pop(context),
    );
  }

  Widget _buildPin(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => PinBloc(context.read<GetRevocationCodeUseCase>()),
      child: PinPage(
        headerBuilder: (context, attemptsLeftInRound, isFinalRound) => PinHeader(
          title: context.l10n.generalConfirmWithPin,
        ),
        onPinValidated: (revocationCode) {
          context.read<ReviewRevocationCodeBloc>().add(ReviewRevocationCodeLoaded(revocationCode));
        },
      ),
    );
  }

  Widget _buildSuccess(BuildContext context, String revocationCode) {
    return TerminalPage(
      title: context.l10n.reviewRevocationCodeScreenSuccessTitle,
      description: context.l10n.reviewRevocationCodeScreenSuccessDescription,
      illustration: Padding(
        padding: const EdgeInsets.fromLTRB(16, 0, 16, 24),
        child: RevocationCodeText(revocationCode: revocationCode),
      ),
      primaryButtonCta: context.l10n.reviewRevocationCodeScreenSuccessCta,
      onPrimaryPressed: () => context.read<ReviewRevocationCodeBloc>().add(const ReviewRevocationCodeRestartFlow()),
      primaryButtonIcon: const Icon(Icons.check_outlined),
    );
  }

  Widget _buildBackButton(BuildContext context) {
    return BlocBuilder<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
      builder: (BuildContext context, ReviewRevocationCodeState state) {
        switch (state) {
          case ReviewRevocationCodeInitial():
            return const BackIconButton();
          case ReviewRevocationCodeProvidePin():
            return BackIconButton(
              onPressed: () => context.read<ReviewRevocationCodeBloc>().add(const ReviewRevocationCodeRestartFlow()),
            );
          case ReviewRevocationCodeSuccess():
            return const SizedBox.shrink();
        }
      },
    );
  }

  Widget _buildCloseButton(BuildContext context) {
    return BlocBuilder<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
      builder: (BuildContext context, ReviewRevocationCodeState state) {
        switch (state) {
          case ReviewRevocationCodeInitial():
          case ReviewRevocationCodeProvidePin():
            return const SizedBox.shrink();
          case ReviewRevocationCodeSuccess():
            return const CloseIconButton();
        }
      },
    );
  }

  FlowProgress? _resolveProgress(BuildContext context) {
    final state = context.watch<ReviewRevocationCodeBloc>().state;
    switch (state) {
      case ReviewRevocationCodeInitial():
        return null;
      case ReviewRevocationCodeProvidePin():
        return const FlowProgress(currentStep: 1, totalSteps: 2);
      case ReviewRevocationCodeSuccess():
        return const FlowProgress(currentStep: 2, totalSteps: 2);
    }
  }
}
