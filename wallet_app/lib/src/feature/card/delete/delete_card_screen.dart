import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repository/card/wallet_card_repository.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/usecase/card/impl/delete_wallet_card_usecase_impl.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/navigator_state_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/page/terminal_page.dart';
import '../../common/widget/button/icon/close_icon_button.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/fake_paging_animated_switcher.dart';
import '../../common/widget/page_illustration.dart';
import '../../common/widget/pin_header.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';
import 'argument/delete_card_screen_argument.dart';
import 'bloc/delete_card_bloc.dart';

class DeleteCardScreen extends StatelessWidget {
  static DeleteCardScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return DeleteCardScreenArgument.fromMap(args! as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [DeleteCardScreenArgument] when opening the DeleteCardScreen');
    }
  }

  const DeleteCardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        automaticallyImplyLeading: false,
        fadeInTitleOnScroll: false,
        actions: [
          const HelpIconButton(),
          CloseIconButton(onPressed: () => _onClose(context)),
        ],
        progress: _resolveProgress(context),
      ),
      body: SafeArea(
        child: BlocBuilder<DeleteCardBloc, DeleteCardState>(
          builder: (context, state) {
            final Widget content = switch (state) {
              DeleteCardInitial() => const SizedBox.shrink(),
              DeleteCardProvidePin(:final attestationId) => _buildPin(context, attestationId),
              DeleteCardSuccess(:final cardTitle) => _buildSuccess(context, cardTitle),
            };
            if (state is DeleteCardInitial || state is DeleteCardProvidePin) return content;
            return FakePagingAnimatedSwitcher(child: content);
          },
        ),
      ),
    );
  }

  /// Creates the usecase inline with the at runtime required [attestationId].
  Widget _buildPin(BuildContext context, String attestationId) {
    return BlocProvider<PinBloc>(
      create: (context) => PinBloc(DeleteWalletCardUseCaseImpl(context.read<WalletCardRepository>(), attestationId)),
      child: PinPage(
        headerBuilder: (context, attemptsLeftInRound, isFinalRound) => PinHeader(
          title: context.l10n.generalConfirmWithPin,
        ),
        onPinValidated: (_) => context.bloc.add(const DeleteCardPinConfirmed()),
      ),
    );
  }

  Widget _buildSuccess(BuildContext context, String cardTitle) {
    return TerminalPage(
      title: context.l10n.deleteCardSuccessPageTitle(cardTitle),
      description: context.l10n.deleteCardSuccessPageDescription,
      illustration: const PageIllustration(asset: WalletAssets.svg_delete_card_success),
      primaryButton: PrimaryButton(
        text: Text(context.l10n.deleteCardSuccessPageToDashboardCta),
        icon: const Icon(Icons.arrow_forward_outlined),
        onPressed: () => Navigator.of(context).resetToDashboard(),
      ),
    );
  }

  void _onClose(BuildContext context) {
    final state = context.bloc.state;
    if (state is DeleteCardSuccess) {
      Navigator.of(context).resetToDashboard();
    } else {
      Navigator.pop(context);
    }
  }

  FlowProgress _resolveProgress(BuildContext context) {
    final state = context.watch<DeleteCardBloc>().state;
    return switch (state) {
      DeleteCardInitial() => const FlowProgress(currentStep: 1, totalSteps: 2),
      DeleteCardProvidePin() => const FlowProgress(currentStep: 1, totalSteps: 2),
      DeleteCardSuccess() => const FlowProgress(currentStep: 2, totalSteps: 2),
    };
  }
}

extension _DeleteCardScreenExtensions on BuildContext {
  DeleteCardBloc get bloc => read<DeleteCardBloc>();
}
